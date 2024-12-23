import type { EnvHashMap } from "@/core";
import Modal from '@@/utils/Modal';

import { create } from "zustand";
import { useEffect, useState } from "react";
import { flush as _flush, TaskAction, receive_state as _receive_state } from "@/core";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';

type SyncState = "SYNCED" | "NOT_SYNCED" | "SYNCING";
interface IStore {
    // EnvHashMap and its actions
    envs: EnvHashMap;
    load: () => Promise<void>;
    flush: () => Promise<void>;

    addVariable: (variable: string) => void;
    deleteVariable: (variable: string) => void;
    appendValue: (variable: string, value: string) => string[];
    modifyValue: (variable: string, index: number, value: string) => string[];
    deleteValue: (variable: string, index: number) => string[];
    orderValue: (variable: string, index: number, direction: "up" | "down") => string[];

    // state management
    syncState: SyncState;
    setSyncState: (state: "SYNCED" | "NOT_SYNCED" | "SYNCING") => void;

    // UI state control
    currentVariable: string;
    switchVariable: (variable: string) => void;
}


const useStore = create<IStore>((set, get) => ({
    envs: {},
    load: async () => {
        let { env, dirty } = await _receive_state();
        set((state) => ({ ...state, envs: env, syncState: dirty ? 'NOT_SYNCED' : 'SYNCED' }));
    },
    flush: async () => {
        set({ syncState: "SYNCING" });
        let { env, dirty } = await _flush();
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
        TaskAction.DelVariable({ variable });
        set((state) => {
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
        return get().envs[variable];
    },

    modifyValue: (variable: string, index: number, value: string) => {
        TaskAction.ModifyValue({ variable, index, old_value: get().envs[variable][index], new_value: value });
        set((state) => {
            state.envs[variable][index] = value;
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
        return get().envs[variable];
    },

    deleteValue: (variable: string, index: number) => {
        TaskAction.DeleteValue({ variable, index, value: get().envs[variable][index] });
        set((state) => {
            state.envs[variable].splice(index, 1);
            return state;
        });
        set({ syncState: "NOT_SYNCED" });
        return get().envs[variable];
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
        return get().envs[variable];
    },

    // UI state control
    currentVariable: "NOTHING",
    switchVariable: (variable: string) => set({ currentVariable: variable }),
}));

// ============
// ==== UI ====
// ============

export default function Main(props: { style?: React.CSSProperties }) {
    const { style } = props;
    const { load } = useStore();

    useEffect(() => {
        load();
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
        const _envKeys = Object.keys(envs).sort();
        setEnvKeys(_envKeys);
        const isFound = _envKeys.find((key) => key === currentVariable);
        if (!isFound) {
            switchVariable(_envKeys[0]);
        }
    }, [envs]);

    const btnDelete = () => {
        _ask(`Are you sure to delete the variable '${currentVariable}'?`).then((res) => {
            if (!res) return;
            deleteVariable(currentVariable);
            switchVariable(envKeys[0]);
        });
    }

    return (
        <>
            <div className="btn-group">
                <button onClick={() => setAdding(true)}>Add</button>
                <button onClick={btnDelete}>Del</button>
            </div>
            <div className="list">
                {envKeys.map((key) => (
                    <div key={key}
                        className={"var-item " + (currentVariable === key ? " active" : "")}
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
    const { syncState, currentVariable } = useStore();

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
                <div>当前应用状态：{stateDom}</div>
                当前选择的环境变量是：<strong>{currentVariable}</strong>
            </div>
        </>
    )
}


function ValueList() {
    const [valueList, setValueList] = useState<string[]>([]);
    const [buffer, setBuffer] = useState<string>("");

    const { flush, envs, currentVariable, appendValue: addValue, modifyValue, deleteValue, orderValue } = useStore();

    const [curEditValIndex, _setEditValIndex] = useState<number>(-1);
    const [isAddValueOpen, _setAddValueOpen] = useState<boolean>(false);
    const setEditValIndex = (index: number) => {
        _setEditValIndex(index);
        _setAddValueOpen(false);
    }
    const setAddValueOpen = (open: boolean) => {
        _setAddValueOpen(open);
        _setEditValIndex(-1);
    }

    useEffect(() => {
        setEditValIndex(-1);
        setAddValueOpen(false);
        setBuffer("");
        setValueList(envs[currentVariable] || []);
    }, [currentVariable]);


    const btnAdd = () => {
        setAddValueOpen(true);
        setBuffer("");
    };
    const btnFlush = async () => await flush();
    const btnRefresh = () => window.location.reload();

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

    const btnFromFS = () => {
        _open({ directory: true, multiple: false }).then((res) => {
            if (!res) return;
            setBuffer(res);
        });
    }

    return (
        <>
            <div className="btn-group">
                <button onClick={btnAdd}>Add</button>
                <button onClick={btnFlush}>Flush</button>
                <button onClick={btnRefresh}>Refresh</button>
                {/* <button onClick={() => { }}>Undo</button>
                <button onClick={() => { }}>Redo</button> */}
            </div>
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
                            <button onClick={() => btnOrder('up')}><Up /></button>
                            <button onClick={() => btnOrder('down')}><Down /></button>

                            <input
                                onChange={(e) => setBuffer(e.currentTarget.value)}
                                placeholder="Enter value"
                                value={buffer}
                            />
                            <button onClick={btnModifyConform}><Checkmark /></button>
                            <button onClick={btnFromFS}><FromFS /></button>
                            <button onClick={btnDelete}><Delete /></button>
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
                    <button onClick={btnAddConform}><Checkmark /></button>
                    <button onClick={btnFromFS}><FromFS /></button>
                </div>
            </div>
        </>
    )
}

import { Checkmark12Filled, Document16Filled, Delete16Filled, ArrowUp12Filled, ArrowDown12Filled } from '@ricons/fluent'
import { Icon } from '@ricons/utils'

function Checkmark() {
    return <Icon size="24"><Checkmark12Filled /></Icon>
}
function FromFS() {
    return <Icon size="24"><Document16Filled /></Icon>
}
function Delete() {
    return <Icon size="24"><Delete16Filled /></Icon>
}
function Up() {
    return <Icon size="24"><ArrowUp12Filled /></Icon>
}
function Down() {
    return <Icon size="24"><ArrowDown12Filled /></Icon>
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





