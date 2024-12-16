import { useEffect, useState } from "react";
import { useStore } from "./store";
import { useEnvStore } from "../core";

export default function EnvList() {
    const [envKeys, setEnvKeys] = useState<string[]>([]);

    const store = useStore();
    const envStore = useEnvStore();

    const { createTask } = envStore;
    const { setEnvVar, setEditingValue, setAdding, setBuffer } = store;

    useEffect(() => {
        setEnvKeys(Object.keys(envStore.envs));
    }, [envStore.envs]);

    return (
        <>
            <h1>Welcome to Tauri + React</h1>
            <div key="left" className="env-var-list">
                {envKeys.map((key) => (
                    <div key={key} className="pointer"
                        onClick={() => {
                            setEnvVar(key)
                            setEditingValue("");
                            setAdding(false);
                            setBuffer("");
                        }}>
                        <strong>{key}</strong>
                    </div>
                ))}
            </div>
        </>
    )
}