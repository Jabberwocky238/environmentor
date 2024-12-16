import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export enum TaskType {
    AddValue,
    ModifyValue,
    DeleteValue,
    AddVariable,
    DeleteVariable,
}

interface Task {
    t: TaskType
    variable: string
}

class AddValueTask implements Task {
    t: TaskType;
    variable: string;
    value: string;

    constructor(t: TaskType, variable: string, value: string) {
        this.t = t;
        this.variable = variable;
        this.value = value;
    }
}

class ModifyValueTask implements Task {
    t: TaskType;
    variable: string;
    oldValue: string;
    newValue: string;

    constructor(t: TaskType, variable: string, oldValue: string, newValue: string) {
        this.t = t;
        this.variable = variable;
        this.oldValue = oldValue;
        this.newValue = newValue;
    }
}

class DeleteValueTask implements Task {
    t: TaskType;
    variable: string;
    value: string;

    constructor(t: TaskType, variable: string, value: string) {
        this.t = t;
        this.variable = variable;
        this.value = value;
    }
}

class AddVariableTask implements Task {
    t: TaskType;
    variable: string;

    constructor(t: TaskType, variable: string) {
        this.t = t;
        this.variable = variable;
    }
}

class DeleteVariableTask implements Task {
    t: TaskType;
    variable: string;

    constructor(t: TaskType, variable: string) {
        this.t = t;
        this.variable = variable;
    }
}

function createTask(t: TaskType, variable: string, value?: string, oldValue?: string): Task {
    switch (t) {
        case TaskType.AddValue:
            return new AddValueTask(t, variable, value!);
        case TaskType.ModifyValue:
            return new ModifyValueTask(t, variable, oldValue!, value!);
        case TaskType.DeleteValue:
            return new DeleteValueTask(t, variable, value!);
        case TaskType.AddVariable:
            return new AddVariableTask(t, variable);
        case TaskType.DeleteVariable:
            return new DeleteVariableTask(t, variable);
    }
}

export class TaskQueue {
    queue: Task[] = []

    push(task: Task) {
        this.queue.push(task);
    }

    remove(task: Task) {
        const index = this.queue.indexOf(task);
        if (index > -1) {
            this.queue.splice(index, 1);
        }
    }

    clear() {
        this.queue = [];
    }

    size(): number {
        return this.queue.length;
    }

    isEmpty(): boolean {
        return this.queue.length === 0;
    }
}

export type EnvHashMap = { [key: string]: string[] };

interface IStore {
    envs: EnvHashMap;
    queue: TaskQueue;
    createTask: (t: TaskType, variable: string, value?: string, oldValue?: string) => Task;
    get_all: () => Promise<EnvHashMap>;
    get_one: (variable: string) => Promise<string[]>;
    set_one: (variable: string, values: string[]) => Promise<string[]>;
}

const useDebugStore = create<IStore>((set, get) => ({
    envs: {
        "var": ["ZQZQ111"],
        "value": ["demo1"],
        "ASLA": ["ASLA1111", "22222222"],
        "ASLB": ["ASLA1111", "11111111111asssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssssss"],
        "ASLC": ["ASLA1111", "1111111111我草泥马1"],
    } as EnvHashMap,
    queue: new TaskQueue(),
    createTask: (t: TaskType, variable: string, value?: string, oldValue?: string) => {
        const task = createTask(t, variable, value, oldValue);
        get().queue.push(task);
        return task;
    },
    get_all: async () => {
        return get().envs;
    },
    get_one: async (variable: string) => {
        return get().envs[variable];
    },
    set_one: async (variable: string, values: string[]) => {
        set((state) => {
            state.envs[variable] = values;
            return state;
        });
        return get().envs[variable];
    },
}));


const useEnvStore = create<IStore>((set, get) => ({
    envs: {},
    queue: new TaskQueue(),
    createTask: (t: TaskType, variable: string, value?: string, oldValue?: string) => {
        const task = createTask(t, variable, value, oldValue);
        set((state) => {
            state.queue.push(task);

            function preResolve(task: Task) {
                switch (task.t) {
                    case TaskType.AddValue:
                        state.envs[task.variable].push((task as AddValueTask).value);
                        break;
                    case TaskType.ModifyValue:
                        const _task = task as ModifyValueTask;
                        const index = state.envs[_task.variable].indexOf(_task.oldValue);
                        state.envs[_task.variable].splice(index, 1, _task.newValue);
                        break;
                    case TaskType.DeleteValue:
                        const _task1 = task as DeleteValueTask;
                        const index1 = state.envs[_task1.variable].indexOf(_task1.value);
                        state.envs[_task1.variable].splice(index1, 1);
                        break;
                    case TaskType.AddVariable:
                        state.envs[task.variable] = [];
                        break;
                    case TaskType.DeleteVariable:
                        delete state.envs[task.variable];
                        break;
                }
            }
            preResolve(task);
            return state;
        })
        console.log(get().queue);
        return task;
    },
    get_all: async () => {
        const envs = await get_all();
        set({ envs });
        return get().envs;
    },
    get_one: async (variable: string) => {
        const values = await get_one(variable);
        return values;
    },
    set_one: async (variable: string, values: string[]) => {
        const _values = await set_one(variable, values);
        set((state) => {
            state.envs[variable] = _values;
            return state;
        });
        return get().envs[variable];
    },
}));

export { useEnvStore as useEnvStore };

async function get_all(): Promise<EnvHashMap> {
    return invoke("get_all");
}

async function get_one(variable: string): Promise<string[]> {
    return invoke("get_one", { var: variable });
}

async function set_one(variable: string, values: string[]): Promise<string[]> {
    const value = values.join(";");
    return invoke("set_one", { var: variable, value });
}


