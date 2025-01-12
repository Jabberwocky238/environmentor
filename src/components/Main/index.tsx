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
import EnvList from "./EnvList";
import ValueList from "./ValueList";

// ============
// ==== UI ====
// ============

export default function Main(props: { style?: React.CSSProperties }) {
    const { style } = props;
    const { load } = useStore();

    useEffect(() => {
        load();
    }, []);

    return (
        <div style={style} className="row">
            <div className="col" style={{ '--col-width': '25%' } as React.CSSProperties}>
                <EnvList></EnvList>
            </div>
            <div className="col" style={{ '--col-width': '75%' } as React.CSSProperties}>
                <Control></Control>
                <ValueList></ValueList>
            </div>
        </div>
    )
}


function Control() {
    const [stateDom, setStateDom] = useState<React.ReactNode>(<StateClean />);
    const { syncState, currentVariable, currentValue } = useStore();

    function StateClean() {
        return <strong style={{ color: 'green' }}>已同步</strong>
    }
    function StateNotSync() {
        return <strong style={{ color: 'orange' }}>未同步</strong>
    }
    function StateSyncing() {
        return <strong style={{ color: 'skyblue' }}>同步中</strong>
    }
    function StateERROR() {
        return <strong style={{ color: 'red' }}>ERROR</strong>
    }

    useEffect(() => {
        if (syncState === "SYNCED") {
            setStateDom(<StateClean />);
        } else if (syncState === "NOT_SYNCED") {
            setStateDom(<StateNotSync />);
        } else if (syncState === "SYNCING") {
            setStateDom(<StateSyncing />);
        } else {
            setStateDom(<StateERROR />);
        }
    }, [syncState]);

    return (
        <>
            <div>
                <div>当前应用状态：{stateDom}</div>
                <div>当前选择的环境变量是：<strong>{currentVariable}</strong></div>
                <div>当前选择的值是: <strong>{currentValue}</strong></div>
            </div>
        </>
    )
}


