import Modal from '@@/utils/Modal';

import { useEffect, useState } from "react";
import { emitter } from "@/core";
import { ask as tauri_ask } from '@tauri-apps/plugin-dialog';
import { INotification } from "@@/utils/Notification";
import { Checkmark } from "@@/utils/Icons";
import { action, useStore } from "./store";

export default function EnvList() {
    const [envKeys, setEnvKeys] = useState<string[]>([]);
    const [buffer, setBuffer] = useState<string>("");
    const [isAdding, setAdding] = useState<boolean>(false);

    const { envs, currentVariable, switchVariable } = useStore();

    useEffect(() => {
        const _envKeys = Object.keys(envs).sort();
        setEnvKeys(_envKeys);
        const isFound = _envKeys.find((key) => key === currentVariable);
        if (!isFound) {
            switchVariable(_envKeys[0]);
        }
    }, [envs]);

    const btnDelete = () => {
        tauri_ask(`Are you sure to delete the variable '${currentVariable}'?`).then((res) => {
            if (!res) return;
            action.variable.remove(currentVariable);
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
                            action.variable.add(buffer);
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