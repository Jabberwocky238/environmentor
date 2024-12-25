import { create } from "zustand";
import { useEffect, useState } from "react";
import { open as _open, ask as _ask } from '@tauri-apps/plugin-dialog';

export default function() {
    return (
        <div className="row">
            <div className="col" style={{ '--col-width': '100%' } as React.CSSProperties}>
                <div className="list">
                    {/* 设置toast显示时间 */}
                        <p>Anaconda</p>
                        <p>启动</p>
                </div>
            </div>
        </div>
    );
}
