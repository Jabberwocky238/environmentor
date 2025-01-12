import type { EnvHashMap } from "@/core";
import { create } from "zustand";
import { flush as _flush, TaskAction, receive_state as _receive_state, undo as _undo } from "@/core";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';

type SyncState = "SYNCED" | "NOT_SYNCED" | "SYNCING";
export interface IStore {
    // EnvHashMap and its actions
    envs: EnvHashMap;
    load: () => Promise<void>;
    flush: () => Promise<void>;
    undo: () => Promise<void>;

    addVariable: (variable: string) => void;
    deleteVariable: (variable: string) => void;
    appendValue: (variable: string, value: string) => void;
    modifyValue: (variable: string, index: number, value: string) => void;
    deleteValue: (variable: string, index: number) => void;
    orderValue: (variable: string, index: number, direction: "up" | "down") => void;

    // state management
    syncState: SyncState;
    setSyncState: (state: "SYNCED" | "NOT_SYNCED" | "SYNCING") => void;

    // UI state control
    currentVariable: string;
    switchVariable: (variable: string) => void;
    currentValue: string;
    switchValue: (value: string) => void;
}

export const useStore = create<IStore>((set, get) => ({
    envs: {},
    load: async () => {
        let { env, dirty } = await _receive_state();
        set((state) => ({ ...state, envs: env, syncState: dirty ? 'NOT_SYNCED' : 'SYNCED' }));
    },
    flush: async () => {
        set({ syncState: "SYNCING" });
        await _flush();
        let { env, dirty } = await _receive_state();
        set((state) => ({ ...state, envs: env, syncState: dirty ? 'NOT_SYNCED' : 'SYNCED' }));
    },
    undo: async () => {
        await _undo();
        let { env, dirty } = await _receive_state();
        set((state) => ({ ...state, envs: env, syncState: dirty ? 'NOT_SYNCED' : 'SYNCED' }));
    },

    // syncState management
    syncState: "SYNCED" as SyncState,
    setSyncState: (state: SyncState) => set({ syncState: state }),

    // env actions
    addVariable: (variable: string) => {
        const _variable = variable.trim().toUpperCase();
        TaskAction.AddVariable({ variable: _variable });
        set((state) => {
            // state.envs[_variable] = [];
            state.envs = { ...state.envs, [_variable]: [] };
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
    },
    deleteVariable: (variable: string) => {
        set((state) => {
            TaskAction.DelVariable({ variable, values: state.envs[variable] });
            delete state.envs[variable];
            state.envs = { ...state.envs };
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
    },

    appendValue: (variable: string, value: string) => {
        TaskAction.AppendValue({ variable, value });
        set((state) => {
            state.envs[variable].push(value);
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
    },

    modifyValue: (variable: string, index: number, value: string) => {
        TaskAction.ModifyValue({ variable, index, old_value: get().envs[variable][index], new_value: value });
        set((state) => {
            state.envs[variable][index] = value;
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
    },

    deleteValue: (variable: string, index: number) => {
        TaskAction.DeleteValue({ variable, index, value: get().envs[variable][index] });
        set((state) => {
            state.envs[variable].splice(index, 1);
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
    },

    orderValue: (variable: string, index: number, direction: "up" | "down") => {
        const new_index = direction === "up" ? index - 1 : index + 1;
        const value = get().envs[variable][index];
        TaskAction.ReorderValue({ variable, index_before: index, index_after: new_index, value });
        set((state) => {
            state.envs[variable].splice(index, 1);
            state.envs[variable].splice(new_index, 0, value);
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
    },

    // UI state control
    currentVariable: "NOTHING",
    switchVariable: (variable: string) => set({ currentVariable: variable }),
    currentValue: "NOTHING",
    switchValue: (value: string) => set({ currentValue: value }),
}));
