import { create } from "zustand";

interface IStore {
    curVar: string;
    isAddingValue: boolean;

    switchVariable: (variable: string) => void;
    setAddingValue: (isAdding: boolean) => void;
}

export const useStore = create<IStore>((set) => ({
    curVar: "NOTHING",
    isAddingValue: false,

    switchVariable: (variable: string) => set({ curVar: variable }),
    setAddingValue: (isAdding: boolean) => set({ isAddingValue: isAdding }),
}));