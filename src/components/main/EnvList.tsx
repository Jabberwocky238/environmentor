import { useEffect, useState } from "react";
import { useStore } from "./store";
import { emitter, useEnv } from "@/core";
import Modal from '@@/utils/Modal';

export default function EnvList() {
    const [envKeys, setEnvKeys] = useState<string[]>([]);
    const [buffer, setBuffer] = useState<string>("");
    const [isAdding, setAdding] = useState<boolean>(false);

    const store = useStore();
    const env = useEnv();
    const { switchVariable } = store;
    const { addVariable } = env;

    useEffect(() => {
        setEnvKeys(Object.keys(env.envs).sort());
        switchVariable(envKeys[0]);
    }, [env.envs]);

    emitter.on("envChange", () => {
        setEnvKeys(Object.keys(env.envs).sort());
        switchVariable(envKeys[0]);
    });

    return (
        <>
            <div>
                <button onClick={() => setAdding(true)}>Add</button>
            </div>
            <div className="var-list">
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
