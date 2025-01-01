import { create } from "zustand";
import { useEffect, useState } from "react";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';
import { FST_get_children as _FST_get_children, FST_scan as _FST_scan, FST_state as _FST_state } from '@/core';

import '@/styles/FSTree.scss';

interface TreeNode {
    name: string;
    absPath: string;
    size: number;
    scriptsCount: number;
    isDir: boolean;
    isAllow: boolean;
    children?: TreeNode[];
}

async function getChildren(path?: string): Promise<TreeNode[]> {
    // console.log("getChildren", path);
    const children = await _FST_get_children(path);
    console.log(children);
    return children.map((child) => ({
        name: child.name,
        absPath: child.abs_path,
        size: child.size,
        scriptsCount: child.scripts_count,
        isDir: child.is_dir,
        isAllow: child.is_allowed,
    }));
}

interface IStore {
    tree: TreeNode[];
    chosen: TreeNode | null;
    choose: (node: TreeNode) => Promise<void>;
    init: () => Promise<void>;
    scan: () => Promise<void>;
    getState: () => Promise<boolean>;
}

const useStore = create<IStore>((set, _) => ({
    tree: [],
    chosen: null,
    choose: async (node) => {
        if (!node.isDir) {
            return set({ chosen: node });
        }
        const children = await getChildren(node.absPath);
        set((state) => {
            node.children = children;
            return { ...state, chosen: node }
        });
    },
    init: async () => {
        const children = await getChildren();
        set({ tree: children, chosen: null });
    },
    scan: async () => {
        const ok = await _ask('This action cannot be reverted. Are you sure?', "Begin to Scan");
        if (!ok) return;
        await _FST_scan();
        const children = await getChildren();
        set({ tree: children, chosen: null });
    },
    getState: async () => {
        return await _FST_state();
    }
}));

function TreeView({ node, style }: { node: TreeNode, style?: React.CSSProperties }) {
    const [open, setOpen] = useState(false);
    const { choose } = useStore();

    const click = () => {
        choose(node);
        setOpen(!open)
    }
    return (
        <ul style={style}>
            <li onClick={click}>{node.name}</li>
            {node.children?.map((child, i) => (
                <TreeView key={i} style={{ display: open ? "block" : "none" }} node={child} />
            ))}
        </ul>
    );
}

function Details({ chosen }: { chosen: TreeNode }) {
    const show_size = (size: number) => {
        if (size < 1024) {
            return <p>{chosen.size.toFixed(3)} bytes</p>;
        }
        size /= 1024;
        if (size < 1024) {
            return <p>{size.toFixed(3)} KB</p>;
        }
        size /= 1024;
        if (size < 1024) {
            return <p>{size.toFixed(3)} MB</p>;
        }
        size /= 1024;
        return <p>{size.toFixed(3)} GB</p>;
    }
    return (
        <div>
            <h2>{chosen.name}</h2>
            <p>{chosen.absPath}</p>
            <p>{chosen.size} bytes</p>
            {show_size(chosen.size)}
            <p>{chosen.scriptsCount} scripts</p>
            <p>{chosen.isDir ? "Directory" : "File"}</p>
            <p>{chosen.isAllow ? "Allow" : "Deny"}</p>
            <div className="btn-group" data-mode="col" data-style="dark">
                {/* <button onClick={() => { }}>Reveal in File Explorer</button>
                <button onClick={() => { }}>Add to Path</button> */}
            </div>
        </div>
    );
}

export default function () {
    const { chosen, tree, scan: _scan, init, getState } = useStore();
    const [showMask, setShowMask] = useState(false);

    useEffect(() => {
        getState().then((busy) => {
            setShowMask(busy);
            if (!busy) {
                init();
            };
        });
    }, []);

    const scan = async () => {
        setShowMask(true);
        await _scan();
        setShowMask(false);
    }

    return (
        <div className="row">
            <div className="col" style={{ '--col-width': '60%' } as React.CSSProperties}>
                <div className="list">
                    <div className="tree-view-container">
                        {tree.map((node, _i) => (
                            <TreeView style={{ paddingInlineStart: 0 }} node={node} />
                        ))}
                    </div>
                </div>
            </div>
            <div className="col" style={{ '--col-width': '40%' } as React.CSSProperties}>
                <div className="btn-group" data-mode="row" data-style="light">
                    <button onClick={scan}>Scan</button>
                </div>
                <div className="list">
                    {chosen && <Details chosen={chosen} />}
                </div>
            </div>
            <div className={showMask ? "FSTreeMask" : "FSTreeMaskDisabled"}>Scanning...please wait...</div>
        </div>
    );
}
