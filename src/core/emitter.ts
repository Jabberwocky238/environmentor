import mitt from "mitt";
import { INotification } from "@@/utils/Notification";

interface IEventType {
    "notification": INotification,
}

type IEmitter = {
    [K in keyof IEventType]: IEventType[K];
};

export const emitter = mitt<IEmitter>();
