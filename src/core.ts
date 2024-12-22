import { invoke } from "@tauri-apps/api/core";

type EnvHashMap = { [key: string]: string[] };
type ReceivedData = { env: EnvHashMap, dirty: boolean };

async function flush(): Promise<ReceivedData> {
    return invoke("flush");
}
async function receive_state(): Promise<ReceivedData> {
    return invoke("send_state")
}


interface ITask {
    'AddVariable': { variable: string },
    'DelVariable': { variable: string },
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

async function task_list(): Promise<any[]> {
    return invoke("task_list")
}

export { flush, TaskAction, receive_state, task_list };
export type { EnvHashMap, ReceivedData };