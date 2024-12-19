import { useEffect, useState } from "react";
import { useEnv } from "@/core";
import { useStore } from "./store";

export default function EnvList() {
    const [valueList, setValueList] = useState<string[]>([]);
    const [buffer, setBuffer] = useState<string>("");
    const [curEditValIndex, setEditValIndex] = useState<number>(-1);

    const env = useEnv();
    const store = useStore();
    const { addValue, modifyValue, deleteValue } = env;
    const { setAddingValue } = store;

    useEffect(() => {
        setEditValIndex(-1);
        setAddingValue(false);
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
                            style={{ display: i === curEditValIndex ? "none" : "block" }}
                            onClick={() => {
                                setEditValIndex(i);
                                console.log(valueList, v)
                                setBuffer(v);
                            }}>{v}</div>

                        <div className="value-item-editing"
                            style={{ display: i === curEditValIndex ? "flex" : "none" }}>
                            <input
                                onChange={(e) => {
                                    setBuffer(e.currentTarget.value)
                                }}
                                placeholder="Enter value"
                                value={buffer}
                            />
                            <button onClick={() => {
                                const newList = modifyValue(store.curVar, i, buffer);
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
                    style={{ display: store.isAddingValue ? "flex" : "none" }}>
                    <input
                        onChange={(e) => setBuffer(e.currentTarget.value)}
                        placeholder="Enter value"
                        value={buffer}
                    />
                    <button onClick={() => {
                        const newList = addValue(store.curVar, buffer);
                        setValueList(newList);
                        setAddingValue(false);
                        setBuffer("");
                    }}>+</button>
                </div>
            </div>
        </>
    )
}