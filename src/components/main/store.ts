import { create } from "zustand";

interface IStore {
    curVar: string;
    isAddValueOpen: boolean;
    buffer: string;
    curEditValIndex: number;

    setEditValIndex: (index: number) => void;
    switchVariable: (variable: string) => void;
    setAddValueOpen: (isAdding: boolean) => void;
    setBuffer: (buffer: string) => void;
}

export const useStore = create<IStore>((set) => ({
    curVar: "NOTHING",
    isAddValueOpen: false,
    buffer: "",
    curEditValIndex: -1,

    setEditValIndex: (index: number) => set({
        curEditValIndex: index,
        isAddValueOpen: false,
    }),
    switchVariable: (variable: string) => set({ curVar: variable }),
    setAddValueOpen: (isAdding: boolean) => set({
        curEditValIndex: -1,
        isAddValueOpen: isAdding,
    }),
    setBuffer: (buffer: string) => set({ buffer }),
}));