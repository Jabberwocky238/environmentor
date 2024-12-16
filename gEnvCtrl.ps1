param (
    [Parameter(Position = 0, Mandatory = $true)][string]$Command = $args[0],
    [Parameter(Position = 1, Mandatory = $false)][string]$With,
    [string]$Name = $null,
    [string]$Value = $null,
    [bool]$Json = $false
)


function Add-EnvValue {
    Backup-EnvValue

    $currentValue = [Environment]::GetEnvironmentVariable($Name, [EnvironmentVariableTarget]::User)
    if ($currentValue) {
        [Environment]::SetEnvironmentVariable($Name, "$currentValue;$Value", [EnvironmentVariableTarget]::User)
    }
    else {
        [Environment]::SetEnvironmentVariable($Name, $Value, [EnvironmentVariableTarget]::User)
    }
    Write-Host "Successful add $Name"
}

function Sub-EnvValue {
    Backup-EnvValue

    $currentValue = [Environment]::GetEnvironmentVariable($Name, [EnvironmentVariableTarget]::User)
    if ($currentValue) {
        $values = $currentValue.Split(';')
        if ($values -contains $Value) {
            $values = $values | Where-Object { $_ -ne $Value }
            $newValue = $values -join ';'
            [Environment]::SetEnvironmentVariable($Name, $newValue, [EnvironmentVariableTarget]::User)
            Write-Host "Successful sub $Name $Value"
        }
        else {
            Write-Warning "No Value $Value in $Name"
        }
    }
    else {
        Write-Warning "No Variable Named $Name "
    }
}

function Clear-EnvValue {
    Backup-EnvValue

    $currentValue = [System.Environment]::GetEnvironmentVariable($Name, [System.EnvironmentVariableTarget]::User)
    if ($currentValue) {
        [Environment]::SetEnvironmentVariable($Name, "", [EnvironmentVariableTarget]::User)
        Write-Host "Successful set $Name"
    }
    else {
        Write-Warning "No Variable Named $Name"
    }
}

[Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User) | ConvertTo-Json
[Environment]::GetEnvironmentVariable("JW_INSTALL", [EnvironmentVariableTarget]::User)

function List {
    $envVars = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User)
    if ($Json) {
        $outputJson = @{}
        $envVars.GetEnumerator() | Sort-Object Name | ForEach-Object {
            $name = $_.Name
            $values = $_.Value -split ';'
            $outputJson[$name] = $values
        }
        Write-Host ( $outputJson | ConvertTo-Json -Depth 100 )
    }
    else {
        $envVars.GetEnumerator() | Sort-Object Name | ForEach-Object {
            $name = $_.Name
            $values = $_.Value -split ';'
            for ($i = 0; $i -lt $values.Length; $i++) {
                Write-Host ("[{0, 20}][{1, 2}]: {2}" -f $name, ($i + 1), $values[$i])
            }
        }
    }
}

function Show-EnvValue {
    $value = [Environment]::GetEnvironmentVariable($Name, [EnvironmentVariableTarget]::User)
    if ($value) {
        Write-Host "Successful show $Name"
        $values = $value -split ';'
        if ($Json) {
            Write-Host ( $values | ConvertTo-Json -Depth 100 )
        }
        else {
            for ($i = 0; $i -lt $values.Length; $i++) {
                Write-Host ("[{0, 2}]: {1}" -f ($i + 1), $values[$i])
            }
        }
    }
    else {
        Write-Warning "No Variable Named $Name "
    }
}

# backup
$BACKUP_DIR = "./EnvVarBackup"

function Backup-EnvValue {
    param (
        [string]$BackupType = "AutoBackup_"
    )    
    $currentDateTime = Get-Date -Format "yyyy_MM_dd_HH_mm_ss"
    $filename = "$BACKUP_DIR/$BackupType$currentDateTime.json"
    # $envVars = @{
    #     "User" = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User)
    #     "Machine" = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::Machine)
    # }
    $envVars = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User)
    $jsonData = $envVars | ConvertTo-Json -Depth 4
    if (-not (Test-Path -Path $BACKUP_DIR)) {
        New-Item -ItemType Directory -Path $BACKUP_DIR
    }
    $jsonData | Out-File -FilePath $filename -Encoding UTF8
    Write-Host "Successful backup at $filename"
}


function ConvertTo-Hashtable {
    param (
        [Parameter(ValueFromPipeline)]$InputObject
    )
    process {
        if ($null -eq $InputObject) { return $null }
        if ($InputObject -is [System.Collections.IEnumerable] -and $InputObject -isnot [string]) {
            $collection = @(
                foreach ($object in $InputObject) {
                    ConvertTo-Hashtable $object
                }
            )
            Write-Output -NoEnumerate $collection
        }
        elseif ($InputObject -is [psobject]) {
            $hashTable = @{}
            foreach ($property in $InputObject.PSObject.Properties) {
                $hashTable[$property.Name] = ConvertTo-Hashtable $property.Value
            }
            $hashTable
        }
        else {
            $InputObject
        }
    }
}


function Compare-EnvValue {
    param (
        [string]$BackupType 
    )
    if ([string]::IsNullOrWhiteSpace($BackupType)) {
        $BackupType = "Manual"
    } 
    if (-not $Json) {
        Write-Output "Compare Mode: [$($BackupType)]"
    }

    $files = Get-ChildItem -Path $BACKUP_DIR -Filter "$BackupType*"
    if ($files.Count -gt 0) {
        $latestFile = $files | Sort-Object CreationTime -Descending | Select-Object -First 1
        if (-not $Json) {
            Write-Output "Found Backup: $($latestFile.Name)"
        }
    }
    else {
        Write-Warning "There is no $BackupType Backup file found."
        return
    }
    # JSON
    $jsonFilePath = "$BACKUP_DIR/$latestFile"
    if (-not (Test-Path $jsonFilePath)) {
        Write-Warning "File $jsonFilePath Not Found."
        return
    }
    $oldEnvVars = Get-Content -Raw -Path $jsonFilePath | ConvertFrom-Json | ConvertTo-Hashtable
    # $currentEnvVars = @{
    #     "User" = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User)
    #     "Machine" = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::Machine)
    # }
    $currentEnvVars = [Environment]::GetEnvironmentVariables([EnvironmentVariableTarget]::User)
    $differences = @()
    $allKeys = $oldEnvVars.Keys + $currentEnvVars.Keys | Sort-Object | Select-Object -Unique
    foreach ($key in $allKeys) {
        if (-not $currentEnvVars.ContainsKey($key)) {
            $differences += [PSCustomObject]@{
                Variable = $key
                Update   = "Delete Variable"
                Value    = $oldEnvVars[$key]
            }
            continue
        } 
        if (-not $oldEnvVars.ContainsKey($key)) {
            $differences += [PSCustomObject]@{
                Variable = $key
                Update   = "New Variable"
                Value    = $currentEnvVars[$key]
            }
            continue
        } 
        $oldValues = $oldEnvVars[$key] -split ';'
        $newValues = $currentEnvVars[$key] -split ';'
        $allValues = $oldValues + $newValues
        foreach ($value in $allValues) {
            if ( -not $oldValues.Contains($value)) {
                $differences += [PSCustomObject]@{
                    Variable = $key
                    Update   = "New Value"
                    Value    = $value
                }
            }
            elseif ( -not $newValues.Contains($value)) {
                $differences += [PSCustomObject]@{
                    Variable = $key
                    Update   = "Delete Value"
                    Value    = $value
                }
            }
        }
    }

    if ($Json) {
        return $differences | ConvertTo-Json -Depth 100
    }
    else {
        if ($differences.Count -eq 0) {
            Write-Host "No changes found."
        }
        return $differences | Format-Table -AutoSize
    }
}

function Restore-EnvValue {
    param (
        [string]$BackupType
    ) 
    if ([string]::IsNullOrWhiteSpace($BackupType)) {
        $BackupType = "Manual"
    }
    Backup-EnvValue -BackupType "RestoreBackup_"
    Write-Output "Restore Mode: [$($BackupType)]"

    $files = Get-ChildItem -Path $BACKUP_DIR -Filter "$BackupType*"
    if ($files.Count -gt 0) {
        $latestFile = $files | Sort-Object CreationTime -Descending | Select-Object -First 1
        Write-Output "Found Backup: $($latestFile.Name)"
    }
    else {
        Write-Output "There is no $BackupType Backup file found."
        return
    }
    # JSON
    $jsonFilePath = "$BACKUP_DIR/$latestFile"

    $differenceKeys = Compare-EnvValue -BackupType $BackupType -OutputType "Raw"
    $differenceKeys = $differenceKeys | ForEach-Object { $_.Variable } | Select-Object -Unique
    $oldEnvVars = Get-Content -Raw -Path $jsonFilePath | ConvertFrom-Json | ConvertTo-Hashtable
    
    if ($differenceKeys) {
        Write-Output "Found Differences: $($differenceKeys.Count)"
        $differenceKeys | ForEach-Object {
            [Environment]::SetEnvironmentVariable($_, $oldEnvVars[$_], [EnvironmentVariableTarget]::User) 
        }
        Write-Output "Restore Complete."
    }
}

# help
function Show-Help {
    $desc = @{
        "add"     = "Add value to environment variable"
        "sub"     = "Sub value from environment variable"
        "show"    = "Show environment variable"
        "clear"   = "Clear environment variable"
        "list"    = "List all user environment variables"
        "cmp"     = "Compare with backup file"
        "backup"  = "Backup environment variables to file"
        "restore" = "Restore environment variables from backup file"
    }
    $usage = @{
        "add"     = "add -Name <Name> -Value <Value>"
        "sub"     = "sub -Name <Name> -Value <Value>"
        "show"    = "show -Name <Name> [-Json $true]"
        "clear"   = "clear -Name <Name>"
        "list"    = "list [-Json $true]"
        "cmp"     = "cmp [Manual|Auto|Restore] [-Json $true]"
        "backup"  = "backup"
        "restore" = "restore [Manual|Auto|Restore]"
    }
    Write-Host "usage: .\gEnvCtrl.ps1 <Command> [Name] [Value]"
    $table = @()
    $keys = $desc.Keys | Sort-Object
    foreach ($key in $keys) {
        $table += [PSCustomObject]@{
            Command     = $key
            Description = ("{0, 7}" -f $desc[$key])
            Usage       = (".\gEnvCtrl.ps1 {0}" -f $usage[$key])
        }
    }
    $table | Format-Table -AutoSize
}

try {
    switch ($Command) {
        "add" { Add-EnvValue }
        "sub" { Sub-EnvValue }
        "show" { Show-EnvValue }
        "clear" { Clear-EnvValue }
        "list" { List }
        "cmp" { Compare-EnvValue -BackupType $With }
        "backup" { Backup-EnvValue -BackupType "ManualBackup_" }
        "restore" { Restore-EnvValue -BackupType $With }
        default { Show-Help }
    }
}
catch {
    Write-Warning "ERROR: $_"
    Show-Help
}