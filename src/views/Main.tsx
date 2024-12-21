import type { EnvHashMap } from "@/core";
import Modal from '@@/utils/Modal';

import { create } from "zustand";
import { useEffect, useState } from "react";
import { flush as _flush, send_state, receive_state as _receive_state } from "@/core";
import { open } from '@tauri-apps/plugin-dialog';

type SyncState = "SYNCED" | "NOT_SYNCED" | "SYNCING";
interface IStore {
    // EnvHashMap and its actions
    envs: EnvHashMap;
    load: () => Promise<void>;
    flush: () => Promise<void>;

    addVariable: (variable: string) => void;
    deleteVariable: (variable: string) => void;
    addValue: (variable: string, value: string) => string[];
    modifyValue: (variable: string, index: number, value: string) => string[];
    deleteValue: (variable: string, index: number) => string[];
    orderValue: (variable: string, index: number, direction: "up" | "down") => string[];

    // state management
    syncState: SyncState;
    setSyncState: (state: "SYNCED" | "NOT_SYNCED" | "SYNCING") => void;

    // UI state control
    currentVariable: string;
    switchVariable: (variable: string) => void;

    isAddValueOpen: boolean;
    setAddValueOpen: (isAdding: boolean) => void;

    buffer: string;
    setBuffer: (buffer: string) => void;

    curEditValIndex: number;
    setEditValIndex: (index: number) => void;
}


const useStore = create<IStore>((set, get) => ({
    envs: {},
    load: async () => {
        let envs: EnvHashMap = await _receive_state();
        set((state) => ({ ...state, envs: envs }), true);
    },
    flush: async () => {
        set({ syncState: "SYNCING" });
        await _flush();
        set({ syncState: "SYNCED" });
    },

    // syncState management
    syncState: "SYNCED" as SyncState,
    setSyncState: (state: SyncState) => set({ syncState: state }),

    // env actions
    addVariable: (variable: string) => {
        const _variable = variable.trim().toUpperCase();
        set((state) => {
            // state.envs[_variable] = [];
            state.envs = { ...state.envs, [_variable]: [] };
            return state;
        });
        send_state(_variable, []);
        set({ syncState: "NOT_SYNCED" });
    },
    deleteVariable: (variable: string) => {
        set((state) => {
            delete state.envs[variable];
            state.envs = { ...state.envs };
            return state;
        });
        send_state(variable, undefined);
        set({ syncState: "NOT_SYNCED" });
    },

    addValue: (variable: string, value: string) => {
        set((state) => {
            state.envs[variable].push(value);
            return state;
        });
        send_state(variable, get().envs[variable]);
        set({ syncState: "NOT_SYNCED" });
        return get().envs[variable];
    },

    modifyValue: (variable: string, index: number, value: string) => {
        set((state) => {
            state.envs[variable][index] = value;
            return state;
        });
        send_state(variable, get().envs[variable]);
        set({ syncState: "NOT_SYNCED" });
        return get().envs[variable];
    },

    deleteValue: (variable: string, index: number) => {
        set((state) => {
            state.envs[variable].splice(index, 1);
            return state;
        });
        send_state(variable, get().envs[variable]);
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
            return state;
        });
        send_state(variable, get().envs[variable]);
        set({ syncState: "NOT_SYNCED" });
        return get().envs[variable];
    },

    // UI state control
    currentVariable: "NOTHING",
    switchVariable: (variable: string) => set({ currentVariable: variable }),

    buffer: "",
    setBuffer: (buffer: string) => set({ buffer }),

    curEditValIndex: -1,
    isAddValueOpen: false,
    setEditValIndex: (index: number) => set({
        curEditValIndex: index,
        isAddValueOpen: false,
    }),
    setAddValueOpen: (isAdding: boolean) => set({
        curEditValIndex: -1,
        isAddValueOpen: isAdding,
    }),
}));

// ============
// ==== UI ====
// ============

export default function Main(props: { style?: React.CSSProperties }) {
    const { style } = props;
    const { load } = useStore();

    useEffect(() => {
        console.log("[mount] Main")
        load();
        return () => {
            console.log("[unmount] Main")
        }
    }, []);

    return (
        <div style={style} className="row">
            <div className="col" style={{ '--col-width': '25%' } as React.CSSProperties}>
                <EnvList></EnvList>
            </div>
            <div className="col" style={{ '--col-width': '75%' } as React.CSSProperties}>
                <Control></Control>
                <ValueList></ValueList>
            </div>
        </div>
    )
}


function EnvList() {
    const [envKeys, setEnvKeys] = useState<string[]>([]);
    const [buffer, setBuffer] = useState<string>("");
    const [isAdding, setAdding] = useState<boolean>(false);

    const { envs, currentVariable, switchVariable, addVariable, deleteVariable } = useStore();

    useEffect(() => {
        console.log("[mount] EnvList")
        setEnvKeys(Object.keys(envs).sort());
        switchVariable(envKeys[0]);
        
        return () => {
            console.log("[unmount] EnvList")
        }
    }, [envs]);

    return (
        <>
            <div>
                <button onClick={() => setAdding(true)}>Add</button>
                <button onClick={() => {
                    deleteVariable(currentVariable);
                    switchVariable(envKeys[0]);
                }}>Del</button>
            </div>
            <div className="list">
                {envKeys.map((key) => (
                    <div key={key} className="var-item"
                        onClick={() => switchVariable(key)}>
                        <strong>{key}</strong>
                    </div>
                ))}
            </div>
            <Modal title='添加新变量' isOpen={isAdding} onClose={() => setAdding(false)}>
                <input
                    onChange={(e) => setBuffer(e.currentTarget.value)}
                    placeholder="Enter value"
                    value={buffer}
                />
                <button onClick={() => {
                    addVariable(buffer);
                    setAdding(false);
                    setBuffer("");
                }}>+</button>
            </Modal>
        </>
    )
}

function Control() {
    const [stateDom, setStateDom] = useState<React.ReactNode>(<StateClean />);
    const { syncState, setSyncState, setAddValueOpen, flush } = useStore();

    const btnAdd = () => setAddValueOpen(true);

    const btnFlush = async () => {
        await flush();
        // setSyncState('SYNCED');
    }

    const btnRefresh = () => {
        window.location.reload();
    }

    // const btnDebug = () => {
    //     task_list().then((res) => {
    //         console.log(res);
    //     });
    // }

    useEffect(() => {
        if (syncState === "SYNCED") {
            setStateDom(<StateClean />);
        } else if (syncState === "NOT_SYNCED") {
            setStateDom(<StateNotSync />);
        } else if (syncState === "SYNCING") {
            setStateDom(<StateSyncing />);
        } else {
            setStateDom(<StateERROR />);
        }
    }, [syncState]);

    return (
        <>
            <div>
                当前应用状态：{stateDom}
                <button onClick={btnAdd}>Add</button>
                <button onClick={btnFlush}>Flush</button>
                <button onClick={btnRefresh}>Refresh</button>
                {/* <button onClick={btnDebug}>Debug</button> */}
                <button onClick={() => { }}>Undo</button>
                <button onClick={() => { }}>Redo</button>
            </div>
        </>
    )
}


function ValueList() {
    const [valueList, setValueList] = useState<string[]>([]);

    const { envs, currentVariable, curEditValIndex, buffer, isAddValueOpen, setAddValueOpen, setBuffer, setEditValIndex, addValue, modifyValue, deleteValue, orderValue } = useStore();

    useEffect(() => {
        setEditValIndex(-1);
        setAddValueOpen(false);
        setBuffer("");
        setValueList(envs[currentVariable] || []);
    }, [currentVariable]);

    const btnOrder = (direction: "up" | "down") => {
        if (direction === "up" && curEditValIndex === 0) return;
        if (direction === "down" && curEditValIndex === valueList.length - 1) return;
        const newList = orderValue(currentVariable, curEditValIndex, direction);
        setValueList(newList);
        setEditValIndex(direction === "up" ? curEditValIndex - 1 : curEditValIndex + 1);
    }

    const btnModifyConform = () => {
        // 如果没有任何变化，就直接退出编辑状态
        if (buffer === valueList[curEditValIndex]) {
            setEditValIndex(-1);
            setBuffer("");
            return;
        }
        const newList = modifyValue(currentVariable, curEditValIndex, buffer);
        setValueList(newList);
        setEditValIndex(-1);
        setBuffer("");
    }

    const btnAddConform = () => {
        const newList = addValue(currentVariable, buffer);
        setValueList(newList);
        setAddValueOpen(false);
        setBuffer("");
    }

    const btnDelete = () => {
        const newList = deleteValue(currentVariable, curEditValIndex);
        setValueList(newList);
        setEditValIndex(-1);
        setBuffer("");
    }

    const btnFromFS = async () => {
        const res = await open({ directory: true, multiple: false });
        if (!res) return;
        console.log(res);
        setBuffer(res || "");
    }

    return (
        <>
            <strong>当前选择的环境变量是：{currentVariable}</strong>
            <div className="list">
                {valueList.map((v, i) => (
                    <>
                        <div className="value-item"
                            style={{ display: i === curEditValIndex ? "none" : "block" }}
                            onClick={() => {
                                setEditValIndex(i);
                                setBuffer(v);
                            }}>{v}</div>

                        <div className="value-item-editing"
                            style={{ display: i === curEditValIndex ? "flex" : "none" }}>
                            <button onClick={() => btnOrder('up')}>↑</button>
                            <button onClick={() => btnOrder('down')}>↓</button>

                            <input
                                onChange={(e) => setBuffer(e.currentTarget.value)}
                                placeholder="Enter value"
                                value={buffer}
                            />
                            <button onClick={btnModifyConform}>Conform</button>
                            <button onClick={btnFromFS}>FromFS</button>
                            <button onClick={btnDelete}>Delete</button>
                        </div>
                    </>
                ))}

                <div className="value-item-editing"
                    style={{ display: isAddValueOpen ? "flex" : "none" }}>
                    <input
                        onChange={(e) => setBuffer(e.currentTarget.value)}
                        placeholder="Enter value"
                        value={buffer}
                    />
                    <button onClick={btnAddConform}>+</button>
                </div>
            </div>
        </>
    )
}

function StateClean() {
    return <strong style={{ color: 'green' }}>已同步</strong>
}

function StateNotSync() {
    return <strong style={{ color: 'orange' }}>未同步</strong>
}

function StateSyncing() {
    return <strong style={{ color: 'skyblue' }}>同步中</strong>
}

function StateERROR() {
    return <strong style={{ color: 'red' }}>ERROR</strong>
}





