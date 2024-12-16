import { create } from "zustand";

interface IStore {
    currentEnvVar: string;
    curEditVal: string;
    isAdding: boolean;
    buffer: string;

    setEnvVar: (currentEnvVar: string) => void;
    setEditingValue: (currentEditingValue: string) => void;
    setAdding: (isAdding: boolean) => void;
    setBuffer: (buffer: string) => void;
}



export const useStore = create<IStore>((set) => ({
    currentEnvVar: "NOTHING",
    curEditVal: "", // currentEditingValue
    isAdding: false,
    buffer: "",
    
    setEnvVar: (currentEnvVar: string) => set({ currentEnvVar }),
    setEditingValue: (currentEditingValue: string) => set({ curEditVal: currentEditingValue }),
    setAdding: (isAdding: boolean) => set({ isAdding }),
    setBuffer: (buffer: string) => set({ buffer }),
}));