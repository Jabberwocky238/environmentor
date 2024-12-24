import { create } from "zustand";
import { useEffect, useState } from "react";
import { flush as _flush, TaskAction, receive_state as _receive_state, undo as _undo, emitter } from "@/core";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';
import { Checkmark } from "@@/utils/Icons";
import { INotification } from "@/components/utils/Notification";
import { EasyStorage } from "@/storage";

export default function Setting() {
    const [buffer, setBuffer] = useState("");
    const [showIndex, setShowIndex] = useState(-1);
    const storage = new EasyStorage();

    const setToastTimeout: React.MouseEventHandler<HTMLButtonElement> = (e) => {
        e.stopPropagation();
        const newTimeout = parseInt(buffer);
        if (isNaN(newTimeout)) {
            emitter.emit("notification", {
                color: "warning",
                title: "输入的不是数字",
                message: "请输入一个数字",
            } satisfies INotification);
            return;
        }
        setBuffer("");
        setShowIndex(-1);
        storage.set("toastTimeout", newTimeout.toString());
    }

    return (
        <div className="row">
            <div className="col" style={{ '--col-width': '100%' } as React.CSSProperties}>
                <div onClick={() => {
                    setShowIndex(0);
                    setBuffer(storage.get("toastTimeout"));
                }} className="list">
                    {/* 设置toast显示时间 */}
                    <div className="item">
                        <p>提示信息显示时间</p>
                        <div style={{ display: showIndex === 0 ? 'none' : 'flex' }}>
                            <p>{storage.get("toastTimeout")} ms</p>
                        </div>
                        <div style={{ display: showIndex === 0 ? 'flex' : 'none', width: '50%' }}
                            className="btn-group">
                            <input
                                onChange={(e) => setBuffer(e.currentTarget.value)}
                                placeholder="Enter value"
                                value={buffer}
                            />
                            <button onClick={setToastTimeout}><Checkmark /></button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
