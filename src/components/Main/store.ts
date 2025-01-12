import { create } from "zustand";
import { API } from "@/core";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';

type SyncState = "SYNCED" | "NOT_SYNCED" | "SYNCING";
export interface IStore {
    // EnvHashMap and its actions
    envs: Record<string, string[]>;
    load: () => Promise<void>;

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
        let { env, dirty } = await API.receive_state();
        set((state) => ({ ...state, envs: env, syncState: dirty ? 'NOT_SYNCED' : 'SYNCED' }));
    },

    // syncState management
    syncState: "SYNCED" as SyncState,
    setSyncState: (state: SyncState) => set({ syncState: state }),

    // UI state control
    currentVariable: "NOTHING",
    switchVariable: (variable: string) => set({ currentVariable: variable }),
    currentValue: "NOTHING",
    switchValue: (value: string) => set({ currentValue: value }),
}));

export namespace action {
    export namespace variable {
        export async function add(variable: string) {
            const _variable = variable.trim().toUpperCase();
            await API.AddVariable({ variable: _variable });
            await useStore.getState().load();
            useStore.setState({
                currentVariable: _variable
            });
        }

        export async function remove(variable: string) {
            await API.DelVariable({ variable, values: useStore.getState().envs[variable] });
            await useStore.getState().load();
        }
    }
    export namespace value {
        export async function append(variable: string, value: string) {
            await API.AppendValue({ variable, value });
            await useStore.getState().load();
        }

        export async function modify(variable: string, index: number, value: string) {
            let oldValue = useStore.getState().envs[variable][index]
            await API.ModifyValue({ variable, index, old_value: oldValue, new_value: value });
            await useStore.getState().load();
        }

        export async function remove(variable: string, index: number) {
            await API.DeleteValue({ variable, index, value: useStore.getState().envs[variable][index] });
            await useStore.getState().load();
        }

        export async function order(variable: string, oldIndex: number, newIndex: number) {
            const value = useStore.getState().envs[variable][oldIndex];
            await API.ReorderValue({ variable, index_before: oldIndex, index_after: newIndex, value });
            await useStore.getState().load();
        }
    }

    export async function flush() {
        useStore.setState({ syncState: "SYNCING" });
        await API.flush();
        await useStore.getState().load();
    }

    export async function undo() {
        await API.undo();
        await useStore.getState().load();
    }

}
