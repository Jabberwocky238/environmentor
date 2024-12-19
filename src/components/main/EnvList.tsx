import React from 'react';
import ReactDOM from 'react-dom';
import { useEffect, useState } from "react";
import { useStore } from "./store";
import { emitter, useEnv } from "@/core";

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
    }, [env.envs]);

    emitter.on("envChange", () => {
        setEnvKeys(Object.keys(env.envs).sort());
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

interface ModalProps {
    title?: string;
    isOpen: boolean;
    onClose: () => void;
    children: React.ReactNode;
}

function Modal(props: ModalProps) {
    const { title, isOpen, onClose, children } = props;

    if (!isOpen) return null;

    return ReactDOM.createPortal(
        <div className="modal">
            <div className="modal-overlay" onClick={onClose} />
            <div className="modal-content">
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: 'center', marginBottom: "10px" }}>
                    <strong className='modal-title'>{title}</strong>
                    <button onClick={onClose}>
                        &times; 
                    </button>
                </div>
                {children}
            </div>
        </div>,
        document.body
    );
};
