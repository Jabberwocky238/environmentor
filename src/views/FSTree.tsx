import { create } from "zustand";
import { useEffect, useState } from "react";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';
import { FST_get_children as _FST_get_children, FST_scan as _FST_scan } from '@/core';

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
        isAllow: child.is_allow,
    }));
}

interface IStore {
    tree: TreeNode[];
    chosen: TreeNode | null;
    choose: (node: TreeNode) => Promise<void>;
    scan: () => Promise<void>;
}

const useStore = create<IStore>((set, get) => ({
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
    scan: async () => {
        const ok = await _ask('This action cannot be reverted. Are you sure?', "Begin to Scan");
        if (!ok) return;
        await _FST_scan();
        const children = await getChildren();
        set({ tree: children, chosen: null });
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
    return (
        <div>
            <h2>{chosen.name}</h2>
            <p>{chosen.absPath}</p>
            <p>{chosen.size} bytes</p>
            <p>{chosen.size / 1024 / 1024} MB</p>
            <p>{chosen.scriptsCount} scripts</p>
            <p>{chosen.isDir ? "Directory" : "File"}</p>
            <p>{chosen.isAllow ? "Allow" : "Deny"}</p>
        </div>
    );
}

export default function () {
    const { chosen, tree, scan } = useStore();

    useEffect(() => {
        getChildren().then((children) => {
            useStore.setState({ tree: children });
        });
    }, []);

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
                <div className="btn-group">
                    <button onClick={scan}>Scan</button>
                    <button onClick={() => {}}>Refresh</button>
                </div>
                <div className="list">
                    {chosen && <Details chosen={chosen} />}
                </div>
            </div>
        </div>
    );
}
