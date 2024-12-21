import { invoke } from "@tauri-apps/api/core";

type EnvHashMap = { [key: string]: string[] };
type ReceivedData = { env: EnvHashMap, dirty: boolean };

async function flush(): Promise<ReceivedData> {
    return invoke("flush");
}
async function send_state(variable: string, values?: string[]): Promise<void> {
    return invoke("receive_state", { variable, values })
}
async function receive_state(): Promise<ReceivedData> {
    return invoke("send_state")
}
async function task_list(): Promise<any[]> {
    return invoke("task_list")
}
async function select_folder(): Promise<string | undefined> {
    return invoke("select_folder")
}

export { flush, send_state, receive_state, task_list, select_folder };
export type { EnvHashMap, ReceivedData };