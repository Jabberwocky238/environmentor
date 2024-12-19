import { useEffect, useState } from "react";
import { useEnv } from "@/core";
import { useStore } from "./store";

export default function EnvList() {
    const [valueList, setValueList] = useState<string[]>([]);

    const env = useEnv();
    const store = useStore();
    const { addValue, modifyValue, deleteValue, orderValue } = env;
    const { setAddValueOpen, setBuffer, setEditValIndex } = store;

    useEffect(() => {
        setEditValIndex(-1);
        setAddValueOpen(false);
        setBuffer("");
        setValueList(env.envs[store.curVar] || []);
    }, [store.curVar]);

    return (
        <>
            <strong>当前选择的环境变量是：{store.curVar}</strong>
            <div className="value-list">
                {valueList.map((v, i) => (
                    <>
                        <div className="value-item"
                            style={{ display: i === store.curEditValIndex ? "none" : "block" }}
                            onClick={() => {
                                setEditValIndex(i);
                                setBuffer(v);
                            }}>{v}</div>

                        <div className="value-item-editing"
                            style={{ display: i === store.curEditValIndex ? "flex" : "none" }}>
                            <button onClick={() => {
                                const newList = orderValue(store.curVar, i, "up");
                                setValueList(newList);
                                setEditValIndex(store.curEditValIndex - 1);
                            }}>↑</button>

                            <button onClick={() => {
                                const newList = orderValue(store.curVar, i, "down");
                                setValueList(newList);
                                setEditValIndex(store.curEditValIndex + 1);
                            }}>↓</button>

                            <input
                                onChange={(e) => setBuffer(e.currentTarget.value)}
                                placeholder="Enter value"
                                value={store.buffer}
                            />

                            <button onClick={() => {
                                // 如果没有任何变化，就直接退出编辑状态
                                if (store.buffer === valueList[i]) {
                                    setEditValIndex(-1);
                                    setBuffer("");
                                    return;
                                }
                                const newList = modifyValue(store.curVar, i, store.buffer);
                                setValueList(newList);
                                setEditValIndex(-1);
                                setBuffer("");
                            }}>Conform</button>

                            <button onClick={() => {
                                const newList = deleteValue(store.curVar, i);
                                setValueList(newList);
                                setEditValIndex(-1);
                                setBuffer("");
                            }}>Delete</button>
                        </div>
                    </>
                ))}

                <div className="value-item-editing"
                    style={{ display: store.isAddValueOpen ? "flex" : "none" }}>
                    <input
                        onChange={(e) => setBuffer(e.currentTarget.value)}
                        placeholder="Enter value"
                        value={store.buffer}
                    />
                    <button onClick={() => {
                        const newList = addValue(store.curVar, store.buffer);
                        setValueList(newList);
                        setAddValueOpen(false);
                        setBuffer("");
                    }}>+</button>
                </div>
            </div>
        </>
    )
}