import { invoke } from "@tauri-apps/api/core";
import mitt from "mitt";
import { INotification } from "./components/utils/Notification";

interface IEventType {
    "notification": INotification,
}

type IEmitter = {
    [K in keyof IEventType]: IEventType[K];
};

export const emitter = mitt<IEmitter>();

interface ISetting {
    "theme": "light" | "dark" | "system";
    "toastTimeout": string;
}

type IEasyStorage = {
    set<K extends keyof ISetting>(key: K, value: ISetting[K]): void,
    get<K extends keyof ISetting>(key: K): ISetting[K],
}

// 存个JB，直接往localstorage里面放
export class EasyStorage implements IEasyStorage {
    constructor() {
        if (!localStorage.getItem("toastTimeout")) {
            localStorage.setItem("toastTimeout", "5000");
        }
        if (!localStorage.getItem("theme")) {
            localStorage.setItem("theme", "system");
        }
    }
    set<K extends keyof ISetting>(key: K, value: ISetting[K]): void {
        localStorage.setItem(key, value.toString());
    }
    get<K extends keyof ISetting>(key: K): ISetting[K] {
        return localStorage.getItem(key) as ISetting[K];
    }
}

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

type EnvHashMap = { [key: string]: string[] };

async function flush(): Promise<void> {
    return invoke("flush");
}
async function receive_state(): Promise<{ env: EnvHashMap, dirty: boolean }> {
    return invoke("send_state")
}
async function undo(): Promise<void> {
    return invoke("undo")
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
export { flush, TaskAction, receive_state, undo, FST_get_children, FST_scan, FST_state };
export type { EnvHashMap };