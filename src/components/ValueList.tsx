import { useEffect, useRef, useState } from "react";
import { useEnvStore, TaskType } from "../core";
import { useStore } from "./store";

export default function EnvList() {
    const store = useStore();
    const envStore = useEnvStore();
    const { setEditingValue, setAdding, setBuffer } = store;
    const { createTask } = envStore;
    
    const addRef = useRef<HTMLInputElement>();

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

                        <form onSubmit={(e) => e.preventDefault()} style={{ display: v === store.curEditVal ? "block" : "none" }}>
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
                            <button>Delete</button>
                        </form>
                    </>
                )) : null}

                <button onClick={() => setAdding(!store.isAdding)}>Add new Value to {store.currentEnvVar}</button>
                <form onSubmit={(e) => e.preventDefault()} style={{ display: store.isAdding ? "block" : "none" }}>
                    <input
                        ref={addRef}
                        onChange={(e) => setBuffer(e.currentTarget.value)}
                        placeholder="Enter value"
                    />
                    <button onClick={() => {
                        createTask(TaskType.AddValue, store.currentEnvVar, store.buffer);
                        setAdding(false);
                        setBuffer("");
                        if (addRef.current) {
                            addRef.current.value = "";
                        }
                    }}>+</button>
                </form>
            </div>
        </>
    )
}