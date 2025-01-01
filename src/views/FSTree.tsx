import { create } from "zustand";
import { useEffect, useState } from "react";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';

import '@/styles/FSTree.scss';

interface TreeNode {
    name: string;
    absPath: string;
    size: number;
    scripts_count: number;
    children?: TreeNode[];
}

function getTree(): TreeNode {
    return {
        name: 'root',
        absPath: '/',
        size: 0,
        scripts_count: 0,
        children: [],
    }
}

interface IStore {
    tree: TreeNode[];
    chosen: TreeNode | null;
    choose: (node: TreeNode) => void;
}

const useStore = create<IStore>((set, get) => ({
    tree: [getTree()],
    chosen: null,
    choose: (node) => {
        set((state) => { 
            node.children = [getTree(), getTree(), getTree()];
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
        </div>
    );
}

export default function () {
    const { chosen, tree } = useStore();

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
