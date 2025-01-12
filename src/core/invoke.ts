
import { invoke } from "@tauri-apps/api/core";
interface ITask {
    'AddVariable': { variable: string },
    'DelVariable': { variable: string, values: string[] },
    'AppendValue': { variable: string, value: string },
    'DeleteValue': { variable: string, index: number, value: string },
    'ModifyValue': { variable: string, index: number, old_value: string, new_value: string },
    'ReorderValue': { variable: string, index_before: number, index_after: number, value: String },
}

interface ITaskAction {
    AddVariable(data: ITask['AddVariable']): Promise<void>,
    DelVariable(data: ITask['DelVariable']): Promise<void>,
    AppendValue(data: ITask['AppendValue']): Promise<void>,
    DeleteValue(data: ITask['DeleteValue']): Promise<void>,
    ModifyValue(data: ITask['ModifyValue']): Promise<void>,
    ReorderValue(data: ITask['ReorderValue']): Promise<void>,
}

const TaskAction: ITaskAction = {
    AddVariable: async (data: ITask['AddVariable']) => invoke("receive_state", { task: { "AddVariable": data } }),
    DelVariable: async (data: ITask['DelVariable']) => invoke("receive_state", { task: { "DelVariable": data } }),
    AppendValue: async (data: ITask['AppendValue']) => invoke("receive_state", { task: { "AppendValue": data } }),
    DeleteValue: async (data: ITask['DeleteValue']) => invoke("receive_state", { task: { "DeleteValue": data } }),
    ModifyValue: async (data: ITask['ModifyValue']) => invoke("receive_state", { task: { "ModifyValue": data } }),
    ReorderValue: async (data: ITask['ReorderValue']) => invoke("receive_state", { task: { "ReorderValue": data } }),
}

async function flush(): Promise<void> {
    return invoke("flush");
}
async function receive_state(): Promise<{ env: { [key: string]: string[] }, dirty: boolean }> {
    return invoke("send_state")
}
async function undo(): Promise<void> {
    return invoke("undo")
}

export const API = {
    ...TaskAction,
    flush,
    receive_state,
    undo,
}


interface TreeNode {
    name: string;
    abs_path: string;
    size: number;
    scripts_count: number;
    is_dir: boolean;
    is_allowed: boolean;
}
async function FST_get_children(absPath?: string): Promise<TreeNode[]> {
    return invoke("FST_children", { absPath });
}
async function FST_scan(): Promise<void> {
    return invoke("FST_scan");
}
async function FST_state(): Promise<boolean> {
    return invoke("FST_state");
}

export { FST_get_children, FST_scan, FST_state };