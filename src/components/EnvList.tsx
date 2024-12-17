import { useEffect, useState } from "react";
import { useStore } from "./store";
import { useEnvStore } from "../core";

export default function EnvList() {
    const [envKeys, setEnvKeys] = useState<string[]>([]);

    const store = useStore();
    const envStore = useEnvStore();

    // const { createTask } = envStore;
    const { setEnvVar, setEditingValue, setAdding, setBuffer } = store;

    useEffect(() => {
        setEnvKeys(Object.keys(envStore.envs).sort());
    }, [envStore.envs]);

    return (
        <>
            <div className="var-list">
                {envKeys.map((key) => (
                    <div key={key} className="var-item"
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