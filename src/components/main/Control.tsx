import { useEffect, useRef, useState } from "react";
import { useEnv } from "@/core";
import { useStore } from "./store";

export default function Control() {
    const refAdd = useRef<HTMLButtonElement>(null);
    const [stateDom, setStateDom] = useState<React.ReactNode>(<StateClean />);
    const env = useEnv();
    const store = useStore();

    const btnAdd = () => {
        store.setAddValueOpen(true);
    }

    const btnFlush = async () => {
        await env.flush();
        env.setSyncState('SYNCED');
    }

    const btnRefresh = () => {
        window.location.reload();
    }

    useEffect(() => {
        switch (env.syncState) {
            case 'SYNCED':
                setStateDom(<StateClean />);
                break;
            case 'NOT_SYNCED':
                setStateDom(<StateNotSync />);
                break;
            case 'SYNCING':
                setStateDom(<StateSyncing />);
                break;
            default:
                setStateDom(<StateERROR />);
                break;
        }

    }, [env.syncState]);

    return (
        <>
            <div>
                当前应用状态：{stateDom}
                <button ref={refAdd} onClick={btnAdd}>Add</button>
                <button onClick={btnFlush}>Flush</button>
                <button onClick={btnRefresh}>Refresh</button>
            </div>
        </>
    )
}

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