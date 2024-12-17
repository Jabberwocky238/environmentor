import { useEnvStore, TaskType } from "../core";
import { useStore } from "./store";

export default function EnvList() {
    const store = useStore();
    const envStore = useEnvStore();
    const { setEditingValue, setAdding, setBuffer } = store;
    const { createTask } = envStore;

    const btnAdd = () => setAdding(!store.isAdding);

    const btnFlush = () => {
        envStore.queue.optimise();
        envStore.queue.execute();
        console.log(envStore.queue);
    }
    return (
        <>
            <div>
                <button onClick={btnAdd}>Add</button>
                <button onClick={btnFlush}>Flush</button>
            </div>
            <strong>当前选择的环境变量是：{store.currentEnvVar}</strong>
            <div className="value-list">
                {envStore.envs[store.currentEnvVar] ? envStore.envs[store.currentEnvVar].map((v) => (
                    <>
                        <div className="value-item"
                            style={{ display: v === store.curEditVal ? "none" : "block" }} onClick={() => {
                                setEditingValue(v);
                                setBuffer(v);
                            }}>{v}</div>

                        <div className="value-item-editing"
                            style={{ display: v === store.curEditVal ? "flex" : "none" }}>
                            <input
                                onChange={(e) => {
                                    setBuffer(e.currentTarget.value)
                                    console.log(e.currentTarget.value)
                                }}
                                placeholder="Enter value"
                                defaultValue={v}
                            />
                            <button onClick={() => {
                                createTask(TaskType.ModifyValue, store.currentEnvVar, store.buffer, v);
                                setEditingValue("");
                                setBuffer("");
                            }}>Conform</button>
                            <button onClick={() => {
                                createTask(TaskType.DeleteValue, store.currentEnvVar, store.buffer);
                                setEditingValue("");
                                setBuffer("");
                            }}>Delete</button>
                        </div>
                    </>
                )) : null}

                <div className="value-item-editing" 
                    style={{ display: store.isAdding ? "flex" : "none" }}>
                    <input
                        onChange={(e) => setBuffer(e.currentTarget.value)}
                        placeholder="Enter value"
                        value={store.buffer}
                    />
                    <button onClick={() => {
                        createTask(TaskType.AddValue, store.currentEnvVar, store.buffer);
                        setAdding(false);
                        setBuffer("");
                    }}>+</button>
                </div>
            </div>
        </>
    )
}