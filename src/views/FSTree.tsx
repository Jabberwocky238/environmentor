import { create } from "zustand";
import { useEffect, useState } from "react";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';
import { FST_get_children as _FST_get_children } from '@/core';

import '@/styles/FSTree.scss';

interface TreeNode {
    name: string;
    absPath: string;
    size: number;
    scripts_count: number;
    children?: TreeNode[];
}

async function getChildren(path?: string): Promise<TreeNode[]> {
    // console.log("getChildren", path);
    const children = await _FST_get_children(path);
    return children.map((child) => ({
        name: child.name,
        absPath: child.abs_path,
        size: child.size,
        scripts_count: child.scripts_count,
    }));
}

interface IStore {
    tree: TreeNode[];
    chosen: TreeNode | null;
    choose: (node: TreeNode) => Promise<void>;
}

const useStore = create<IStore>((set, get) => ({
    tree: [],
    chosen: null,
    choose: async (node) => {
        const children = await getChildren(node.absPath);
        set((state) => {
            node.children = children;
            return { ...state, chosen: node }
        });
    },
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
            <p>{chosen.scripts_count} scripts</p>
        </div>
    );
}

export default function () {
    const { chosen, tree } = useStore();

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
                <div className="list">
                    {chosen && <Details chosen={chosen} />}
                </div>
            </div>
        </div>
    );
}
