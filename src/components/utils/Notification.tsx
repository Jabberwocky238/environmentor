import { useEffect, useState } from "react";
import './notification.scss';
import { emitter } from "@/core";

export interface INotification {
    color: 'success' | 'error' | 'warning' | 'info';
    timestamp: number;
    title?: string;
    message: string;
}

function NotificationItem(props: { notification: INotification }) {
    const { notification: n } = props;

    const Type = () => {
        switch (n.color) {
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

    return (
        <div key={`${n.title}${n.timestamp}`} className="notification-item">
            <div className="notification-title"><Type /> {n.title}</div>
            <div className="notification-message">{n.message}</div>
        </div>
    );
}

export default function Notification() {
    const [notifications, setNotifications] = useState<INotification[]>([]);
    const [notificationCount, setNotificationCount] = useState(0);

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
            setNotificationCount((count) => count + 1);
            setTimeout(() => {
                setNotifications((notifications) => notifications.slice(1));
                setNotificationCount((count) => count - 1);
            }, 1000);
        })
        return () => {
            emitter.off("notification");
        };
    }, []);

    return (
        <div className="notification">
            {notifications.map((n) => (
                <NotificationItem notification={n} />
            ))}
        </div>
    );
}