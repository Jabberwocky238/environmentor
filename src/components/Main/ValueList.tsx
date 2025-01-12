import '@/styles/Main.scss';

import { MouseEventHandler, useEffect, useState } from "react";
import { flush as _flush, receive_state as _receive_state, undo as _undo, emitter } from "@/core";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';
import { Checkmark, Delete, Down, FromFS, Up } from "@@/utils/Icons";
import { useStore } from "./store";

function ValueListButtonGroup() {
    const { syncState } = useStore();

    return (
        <div className="btn-group" data-mode="row" data-style="light">
            <button onClick={btnAdd}>Add</button>
            <button onClick={btnFlush}>Flush</button>
            <button onClick={btnRefresh}>Refresh</button>
            <button onClick={btnUndo} disabled={
                syncState === "SYNCING" || syncState === "SYNCED"
            }>Undo</button>
        </div>
    )
}

export default function ValueList() {
    const [buffer, setBuffer] = useState<string>("");

    const { flush, undo, envs, currentVariable, syncState, appendValue, modifyValue, deleteValue, orderValue } = useStore();

    const [curEditValIndex, _setEditValIndex] = useState<number>(-1);
    const [isAddValueOpen, _setAddValueOpen] = useState<boolean>(false);
    const setEditValIndex = (index: number) => {
        _setEditValIndex(index);
        _setAddValueOpen(false);
    }
    const setAddValueOpen = (open: boolean) => {
        _setAddValueOpen(open);
        _setEditValIndex(-1);
    }

    useEffect(() => {
        setEditValIndex(-1);
        setAddValueOpen(false);
        setBuffer("");
    }, [currentVariable]);


    const btnAdd = () => {
        setAddValueOpen(true);
        setBuffer("");
    };
    const btnFlush = async () => await flush();
    const btnRefresh = () => window.location.reload();
    const btnUndo = async () => {
        await undo();
        setEditValIndex(-1);
    };

    const btnOrder = (direction: "up" | "down") => {
        if (direction === "up" && curEditValIndex === 0) return;
        if (direction === "down" && curEditValIndex === envs[currentVariable].length - 1) return;
        orderValue(currentVariable, curEditValIndex, direction);
        setEditValIndex(direction === "up" ? curEditValIndex - 1 : curEditValIndex + 1);
    }

    const btnModifyConform = () => {
        // 如果没有任何变化，就直接退出编辑状态
        if (buffer === envs[currentVariable][curEditValIndex]) {
            setEditValIndex(-1);
            setBuffer("");
            return;
        }
        modifyValue(currentVariable, curEditValIndex, buffer);
        setEditValIndex(-1);
        setBuffer("");
    }

    const btnAddConform = () => {
        appendValue(currentVariable, buffer);
        setAddValueOpen(false);
        setBuffer("");
    }

    const btnDelete = () => {
        deleteValue(currentVariable, curEditValIndex);
        setEditValIndex(-1);
        setBuffer("");
    }

    const btnFromFS = () => {
        _open({ directory: true, multiple: false }).then((res) => {
            if (!res) return;
            setBuffer(res);
        });
    }

    let startX = 0;
    let startY = 0;
    let startMouseX = 0;
    let startMouseY = 0;

    const mouseDown: MouseEventHandler<HTMLDivElement> = (e: React.MouseEvent) => {
        console.log(e);
        const draggable = e.currentTarget!;

        e.currentTarget.addEventListener('mousedown', (e) => {
            startX = draggable.offsetLeft;
            startY = draggable.offsetTop;
            startMouseX = e.clientX;
            startMouseY = e.clientY;
            console.log(startX, startY, startMouseX, startMouseY);

            document.addEventListener('mousemove', mouseMove);
            document.addEventListener('mouseup', mouseUp);
        });

        function mouseMove(e: MouseEvent) {
            const dx = e.clientX - startMouseX;
            const dy = e.clientY - startMouseY;
            draggable.style.left = `${startX + dx}px`;
            draggable.style.top = `${startY + dy}px`;
        }

        function mouseUp() {
            document.removeEventListener('mousemove', mouseMove);
            document.removeEventListener('mouseup', mouseUp);
        }
    }

    return (
        <>
            <div className="btn-group" data-mode="row" data-style="light">
                <button onClick={btnAdd}>Add</button>
                <button onClick={btnFlush}>Flush</button>
                <button onClick={btnRefresh}>Refresh</button>
                <button onClick={btnUndo} disabled={
                    syncState === "SYNCING" || syncState === "SYNCED"
                }>Undo</button>
            </div>
            <div className="list">
                {envs[currentVariable] && envs[currentVariable].map((v, i) => (
                    <>
                        <div className="item"
                            style={{ display: i === curEditValIndex ? "none" : "block" }}
                            onClick={() => {
                                setEditValIndex(i);
                                setBuffer(v);
                            }}
                            onMouseDown={mouseDown}
                            draggable
                        >
                            <p>{v}</p>
                        </div>

                        <div className="item editing"
                            style={{ display: i === curEditValIndex ? "flex" : "none" }}>
                            <button onClick={() => btnOrder('up')}><Up /></button>
                            <button onClick={() => btnOrder('down')}><Down /></button>

                            <input
                                onChange={(e) => setBuffer(e.currentTarget.value)}
                                placeholder="Enter value"
                                value={buffer}
                            />
                            <button onClick={btnModifyConform}><Checkmark /></button>
                            <button onClick={btnFromFS}><FromFS /></button>
                            <button onClick={btnDelete}><Delete /></button>
                        </div>
                    </>
                ))}

                <div className="item editing"
                    style={{ display: isAddValueOpen ? "flex" : "none" }}>
                    <input
                        onChange={(e) => setBuffer(e.currentTarget.value)}
                        placeholder="Enter value"
                        value={buffer}
                    />
                    <button onClick={btnAddConform}><Checkmark /></button>
                    <button onClick={btnFromFS}><FromFS /></button>
                </div>
            </div>
        </>
    )
}






