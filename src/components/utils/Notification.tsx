import { useEffect, useState } from "react";
import './notification.scss';
import { emitter, EasyStorage } from "@/core";

export interface INotification {
    color: 'success' | 'error' | 'warning' | 'info';
    title?: string;
    message: string;
}

const Type = (props: { color: string }) => {
    const color = props.color;
    switch (color) {
        case 'success':
            return <span style={{ color: 'lightgreen' }}>Success</span>;
        case 'error':
            return <span style={{ color: 'red' }}>Error</span>;
        case 'warning':
            return <span style={{ color: 'orange' }}>Warning</span>;
        case 'info':
            return <span style={{ color: 'white' }}>Info</span>;
        default:
            return <span style={{ color: 'white' }}>Info</span>
    }
}

export default function Notification() {
    const [notifications, setNotifications] = useState<INotification[]>([]);
    const [handlers, setHandlers] = useState<NodeJS.Timeout[]>([]);

    const storage = new EasyStorage();
    
    const createTimeout = () => {
        const time = parseInt(storage.get("toastTimeout"));
        return setTimeout(() => {
            setNotifications((notifications) => notifications.slice(1));
            setHandlers((handlers) => handlers.slice(1));
        }, time);
    }

    useEffect(() => {
        emitter.on("notification", (n) => {
            console.log("[Notification useEffect] notification", n);
            
            const handler = createTimeout();
            setNotifications((notifications) => [...notifications, n]);
            setHandlers([...handlers, handler]);
        })
        return () => {
            emitter.off("notification");
        };
    }, []);

    return (
        <div className="notification">
            {notifications.map((n, i) => (
                <div key={`${n.title}${Date.now()}`} className="notification-item">
                    <div className="notification-titlebar">
                        <Type color={n.color} />
                        {new Date().toLocaleDateString() + " " + new Date().toLocaleTimeString()}
                        <button onClick={() => {
                            clearTimeout(handlers[i]);
                            setNotifications((notifications) => notifications.filter((_, index) => index !== i));
                            setHandlers((handlers) => handlers.filter((_, index) => index !== i));
                        }}>x</button>
                    </div>
                    {n.title && <div className="notification-title">{n.title}</div>}
                    <div className="notification-message">{n.message}</div>
                </div>
            ))}
        </div>
    );
}