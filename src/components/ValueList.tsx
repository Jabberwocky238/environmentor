import { useEffect, useRef, useState } from "react";
import { useEnvStore, TaskType } from "../core";
import { useStore } from "./store";

export default function EnvList() {
    const store = useStore();
    const envStore = useEnvStore();
    const { setEditingValue, setAdding, setBuffer } = store;
    const { createTask } = envStore;
    
    return (
        <>
            <strong>当前选择的环境变量是：{store.currentEnvVar}</strong>
            <div className="env-var-list">
                {envStore.envs[store.currentEnvVar] ? envStore.envs[store.currentEnvVar].map((v) => (
                    <>
                        <div id={Math.random().toString()} className="pointer"
                            style={{ display: v === store.curEditVal ? "none" : "block" }} onClick={() => {
                                setEditingValue(v);
                            }}>{v}</div>

                        <div style={{ display: v === store.curEditVal ? "block" : "none" }}>
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

                <button onClick={() => setAdding(!store.isAdding)}>Add new Value to {store.currentEnvVar}</button>
                <div style={{ display: store.isAdding ? "block" : "none" }}>
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