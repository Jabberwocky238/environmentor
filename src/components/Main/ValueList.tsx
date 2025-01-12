import { CSSProperties, MouseEventHandler, useEffect, useState } from "react";
import { Checkmark, Delete, FromFS } from "@@/utils/Icons";
import {
    useStore as useMainStore,
    action
} from "./store";
import { create } from "zustand";
import {
    open as tauri_open
} from '@tauri-apps/plugin-dialog';
import { DragDropContext, Draggable, Droppable, OnDragEndResponder } from "react-beautiful-dnd";
import { match } from "@/core";

namespace utils {
    interface _IStore {
        buffer: string;
        curEditValIndex: number;
        isBlockOpen: boolean;
    }

    export const useStore = create<_IStore>((set, get) => ({
        buffer: "",
        curEditValIndex: -1,
        isBlockOpen: false,
    }));

    export function setBuffer(buffer: string) {
        useStore.setState({ buffer });
    }
    export function setEditValIndex(index: number) {
        useStore.setState({ curEditValIndex: index });
    }
    export function setBlockOpen(open: boolean) {
        useStore.setState({ isBlockOpen: open });
    }
}

namespace btn {
    export namespace control {
        export function add() {
            utils.setBlockOpen(true);
            utils.setBuffer("");
        }
        export async function flush() {
            await action.flush();
        }
        export function refresh() {
            window.location.reload();
        }
        export async function undo() {
            await action.undo();
            utils.setEditValIndex(-1);
        }
    }

    export namespace item {
        export function add() {
            const { buffer } = utils.useStore.getState();
            const { currentVariable } = useMainStore.getState();
            action.value.append(currentVariable, buffer);
            utils.setBlockOpen(false);
            utils.setBuffer("");
        }
        export function order(newIndex: number) {
            const { curEditValIndex } = utils.useStore.getState();
            const { currentVariable } = useMainStore.getState();
            action.value.order(currentVariable, curEditValIndex, newIndex);
            utils.setEditValIndex(newIndex);
        }
        export function modify() {
            const { buffer, curEditValIndex } = utils.useStore.getState();
            const { envs, currentVariable } = useMainStore.getState();
            utils.setBlockOpen(false);
            utils.setEditValIndex(-1);
            utils.setBuffer("");
            if (buffer === envs[currentVariable][curEditValIndex]) {
                return;
            }
            action.value.modify(currentVariable, curEditValIndex, buffer);
        }
        export function remove() {
            const { curEditValIndex } = utils.useStore.getState();
            const { currentVariable } = useMainStore.getState();
            action.value.remove(currentVariable, curEditValIndex);
            utils.setEditValIndex(-1);
            utils.setBuffer("");
        }
        export async function fromFS() {
            const res = await tauri_open({ directory: true, multiple: false });
            if (!res) return;
            utils.setBuffer(res);
        }
    }
}

function ValueListButtonGroup() {
    const { syncState } = useMainStore();
    return (
        <div className="btn-group" data-mode="row" data-style="light">
            <button onClick={btn.control.add}>Add</button>
            <button onClick={btn.control.flush}>Flush</button>
            <button onClick={btn.control.refresh}>Refresh</button>
            <button onClick={btn.control.undo} disabled={
                syncState === "SYNCING" || syncState === "SYNCED"
            }>Undo</button>
        </div>
    )
}

function InfoBlock() {
    const { currentValue, currentVariable } = useMainStore();
    const { isBlockOpen, buffer, curEditValIndex } = utils.useStore();

    const check = () => {
        if (curEditValIndex === -1) {
            btn.item.add();
        } else {
            btn.item.modify();
        }
    }

    const output = () => {
        if (curEditValIndex === -1) {
            return match(currentVariable)?.map((item) => (
                <p key={item}>{item}</p>
            )) ?? "unknown"
        } else {
            return match(currentValue)?.map((item) => (
                <p key={item}>{item}</p>
            )) ?? "unknown"
        }
    }

    return (
        <div className="block" style={{ display: isBlockOpen ? "flex" : "none" }}>
            <h3>Infomation</h3>
            <div className="btn-group" data-mode="row" data-style="dark">
                <input
                    onChange={(e) => utils.setBuffer(e.currentTarget.value)}
                    placeholder="Enter value"
                    value={buffer}
                />
                <button onClick={check}><Checkmark /></button>
                <button onClick={btn.item.fromFS}><FromFS /></button>
                <button onClick={btn.item.remove}><Delete /></button>
            </div>
            <div style={{
                padding: "0.5rem",
            }}>{output()}</div>
        </div>
    )
}


const getItemStyle = (isDragging: boolean, draggableStyle?: CSSProperties): CSSProperties => ({
    borderColor: isDragging ? 'var(--color-active)' : 'transparent',
    ...draggableStyle,
});

const getListStyle = (isDraggingOver: boolean): CSSProperties => ({
    // background: isDraggingOver ? 'lightblue' : 'lightgrey',
});
const genId = (item: string) => ({
    value: item,
    id: `${item}${Math.random()}`
});
type WithId = ReturnType<typeof genId>;

function ValueDraggableList() {
    const { envs, currentVariable, switchValue } = useMainStore();
    const [state, setState] = useState((envs[currentVariable] ?? []).map(genId));

    useEffect(() => {
        utils.setEditValIndex(-1);
        utils.setBlockOpen(false);
        utils.setBuffer("");
        switchValue("");
        setState((envs[currentVariable] ?? []).map(genId));
    }, [currentVariable]);

    const updateColor = (index: number) => {
        const collection = document.getElementById("ValueDraggableList")?.getElementsByClassName("item") ?? [];
        for (let i = 0; i < collection.length; i++) {
            if (i === index) {
                collection[i].classList.add("active")
            } else {
                collection[i].classList.remove("active");
            }
        }
    }
    const reorder = (list: WithId[], startIndex: number, endIndex: number) => {
        const result = Array.from(list);
        const [removed] = result.splice(startIndex, 1);
        result.splice(endIndex, 0, removed);
        return result;
    }
    const onDragEnd: OnDragEndResponder = (result) => {
        if (!result.destination) {
            return;
        }
        if (result.destination.index === result.source.index) {
            return;
        }
        action.value.order(currentVariable, result.source.index, result.destination.index);
        setState(reorder(state, result.source.index, result.destination!.index));
        utils.setEditValIndex(result.destination!.index);
    }
    const createOnClick: (i: number) => MouseEventHandler<HTMLDivElement> = (index) => (e) => {
        e.stopPropagation();
        utils.setEditValIndex(index);
        utils.setBlockOpen(true);
        utils.setBuffer(envs[currentVariable][index]);
        updateColor(index);
        switchValue(envs[currentVariable][index]);
    };

    const clearEdit = () => {
        utils.setEditValIndex(-1);
        utils.setBlockOpen(false);
        utils.setBuffer("");
        updateColor(-1);
        switchValue("");
    }

    return (
        <DragDropContext onDragEnd={onDragEnd}>
            <Droppable droppableId="lis888t">
                {(provided, snapshot) => (
                    <div ref={provided.innerRef}
                        className="list"
                        id="ValueDraggableList"
                        style={getListStyle(snapshot.isDraggingOver)}
                        {...provided.droppableProps}
                    >
                        {state.map((item, index: number) => (
                            <Draggable draggableId={item.id} index={index} key={item.id}>
                                {(provided, snapshot) => (
                                    <div
                                        className="item"
                                        ref={provided.innerRef}
                                        {...provided.draggableProps}
                                        {...provided.dragHandleProps}
                                        style={getItemStyle(snapshot.isDragging, provided.draggableProps.style)}
                                        onClick={createOnClick(index)}
                                    >
                                        <p>{item.value}</p>
                                    </div>
                                )}
                            </Draggable>
                        ))}
                        <div onClick={clearEdit} style={{ flexGrow: 1 }}></div>
                        {provided.placeholder as JSX.Element}
                    </div>
                )}
            </Droppable>
        </DragDropContext>
    )
}

export default function ValueList() {
    return (
        <>
            <ValueListButtonGroup />
            <ValueDraggableList />
            <InfoBlock />
        </>
    )
}






