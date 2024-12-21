import { invoke } from "@tauri-apps/api/core";

type EnvHashMap = { [key: string]: string[] };

async function flush(): Promise<void> {
    return invoke("flush");
}
async function send_state(variable: string, values?: string[]): Promise<void> {
    return invoke("receive_state", { variable, values })
}
async function receive_state(): Promise<EnvHashMap> {
    return invoke("send_state")
}
async function task_list(): Promise<any[]> {
    return invoke("task_list")
}

export { flush, send_state, receive_state, task_list };
export type { EnvHashMap };