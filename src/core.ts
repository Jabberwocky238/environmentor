import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";
import mitt from "mitt";

export const emitter = mitt();
export type EnvHashMap = { [key: string]: string[] };
export type SyncState = "SYNCED" | "NOT_SYNCED" | "SYNCING";

interface IStore {
    envs: EnvHashMap;
    load: () => void;
    flush: () => Promise<void>;

    // state management
    syncState: SyncState;
    setSyncState: (state: "SYNCED" | "NOT_SYNCED" | "SYNCING") => void;

    // 所有面板行为
    addVariable: (variable: string) => void;
    deleteVariable: (variable: string) => void;

    addValue: (variable: string, value: string) => string[];
    modifyValue: (variable: string, index: number, value: string) => string[];
    deleteValue: (variable: string, index: number) => string[];
    orderValue: (variable: string, index: number, direction: "up" | "down") => string[];
}

export const useEnv = create<IStore>((set, get) => ({
    envs: {},
    load: async () => {
        let old_state: EnvHashMap = await load();
        set({ envs: old_state });
    },
    flush: async () => {
        set({ syncState: "SYNCING" });
        await flush();
        set({ syncState: "SYNCED" });
    },

    // state management
    syncState: "SYNCED" as SyncState,
    setSyncState: (state: SyncState) => set({ syncState: state }),

    // 所有面板行为
    addVariable: (variable: string) => {
        const _variable = variable.trim().toUpperCase();
        set((state) => {
            state.envs[_variable] = [];
            send_state(_variable, []);
            return state;
        });
        emitter.emit("envChange");
        set({ syncState: "NOT_SYNCED" });
    },
    deleteVariable: (variable: string) => {
        set((state) => {
            delete state.envs[variable];
            send_state(variable, undefined);
            return state;
        });
        emitter.emit("envChange");
        set({ syncState: "NOT_SYNCED" });
    },

    addValue: (variable: string, value: string) => {
        set((state) => {
            state.envs[variable].push(value);
            send_state(variable, state.envs[variable]);
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
        return get().envs[variable];
    },

    modifyValue: (variable: string, index: number, value: string) => {
        set((state) => {
            state.envs[variable][index] = value;
            send_state(variable, state.envs[variable]);
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
        return get().envs[variable];
    },

    deleteValue: (variable: string, index: number) => {
        set((state) => {
            state.envs[variable].splice(index, 1);
            send_state(variable, state.envs[variable]);
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
        return get().envs[variable];
    },

    orderValue: (variable: string, index: number, direction: "up" | "down") => {
        set((state) => {
            const value = state.envs[variable][index];
            state.envs[variable].splice(index, 1);
            if (direction === "up") {
                state.envs[variable].splice(index - 1, 0, value);
            } else {
                state.envs[variable].splice(index + 1, 0, value);
            }
            send_state(variable, state.envs[variable]);
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
        return get().envs[variable];
    }
}));

async function init(): Promise<EnvHashMap> {
    return invoke("load");
}
async function flush(): Promise<void> {
    return invoke("flush");
}
async function send_state(variable: string, values?: string[]): Promise<void> {
    return invoke("receive_state", { variable, values })
}
async function receive_state(): Promise<EnvHashMap> {
    return invoke("send_state")
}
export async function task_list(): Promise<any[]> {
    return invoke("task_list")
}
