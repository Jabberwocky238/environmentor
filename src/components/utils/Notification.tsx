import { useEffect, useState } from "react";
import './notification.scss';
import { emitter } from "@/core";

export interface INotification {
    color: 'success' | 'error' | 'warning' | 'info';
    timestamp: number;
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

    useEffect(() => {
        emitter.on("notification", (n: any) => {
            const notification = n as INotification;
            console.log("[Notification useEffect] notification", n);
            // const notification: INotification = {
            //     color: 'error',
            //     timestamp: Date.now(),
            //     title: 'Title',
            //     message: 'This is a notification'
            // };
            setNotifications((notifications) => [...notifications, notification]);

            const handler = setTimeout(() => {
                setNotifications((notifications) => notifications.slice(1));
                setHandlers((handlers) => handlers.slice(1));
            }, 3000);
            setHandlers([...handlers, handler]);
        })
        return () => {
            emitter.off("notification");
        };
    }, []);

    return (
        <div className="notification">
            {notifications.map((n, i) => (
                <div key={`${n.title}${n.timestamp}`} className="notification-item">
                    <div className="notification-titlebar">
                        <Type color={n.color} />
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