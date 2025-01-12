import type { EnvHashMap } from "@/core";
import Modal from '@@/utils/Modal';
import '@/styles/Main.scss';

import { create } from "zustand";
import { MouseEventHandler, useEffect, useState } from "react";
import { flush as _flush, TaskAction, receive_state as _receive_state, undo as _undo, emitter } from "@/core";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';
import { INotification } from "@@/utils/Notification";
import { Checkmark, Delete, Down, FromFS, Up } from "@@/utils/Icons";
import { useStore } from "./store";

export default function EnvList() {
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

    const btnClose = () => {
        setAdding(false);
        setBuffer("");
    }

    return (
        <>
            <div className="btn-group" data-mode="row" data-style="light">
                <button onClick={() => setAdding(true)}>Add</button>
                <button onClick={btnDelete}>Del</button>
            </div>
            <div className="list">
                {envKeys.map((key) => (
                    <div key={key}
                        className={"item " + (currentVariable === key ? "active" : "")}
                        onClick={() => switchVariable(key)}>
                        <strong>{key}</strong>
                    </div>
                ))}
            </div>
            <Modal title='添加新变量' isOpen={isAdding} onClose={btnClose}>
                <div className="btn-group">
                    <input
                        onChange={(e) => setBuffer(e.currentTarget.value)}
                        placeholder="Enter value"
                        value={buffer}
                    />
                    <button onClick={() => {
                        if (isEnglishAndNumbers(buffer)) {
                            addVariable(buffer);
                            setAdding(false);
                            setBuffer("");
                        } else {
                            emitter.emit("notification", {
                                color: "error",
                                title: "变量不完全是数字和英文",
                                message: "将会导致操作系统不可预测的行为，不建议这样做，如果您必须如此，请使用操作系统自带的工具手动修改"
                            } satisfies INotification);
                        }
                    }}><Checkmark /></button>
                </div>
            </Modal>
        </>
    )
}

function isEnglishAndNumbers(str: string) {
    const regex = /^[a-zA-Z0-9]+$/;
    return regex.test(str);
}