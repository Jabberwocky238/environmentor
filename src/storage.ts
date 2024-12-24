interface ISetting {
    "theme": "light" | "dark" | "system";
    "toastTimeout": string;
}

type IEasyStorage = {
    set<K extends keyof ISetting>(key: K, value: ISetting[K]): void,
    get<K extends keyof ISetting>(key: K): ISetting[K],
}

// 存个JB，直接往localstorage里面放
export class EasyStorage implements IEasyStorage {
    constructor() {
        if (!localStorage.getItem("toastTimeout")) {
            localStorage.setItem("toastTimeout", "5000");
        }
        if (!localStorage.getItem("theme")) {
            localStorage.setItem("theme", "system");
        }
    }
    set<K extends keyof ISetting>(key: K, value: ISetting[K]): void {
        localStorage.setItem(key, value.toString());
    }
    get<K extends keyof ISetting>(key: K): ISetting[K] {
        return localStorage.getItem(key) as ISetting[K];
    }
}