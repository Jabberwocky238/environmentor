import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

export enum TaskType {
    AddValue,
    ModifyValue,
    DeleteValue,
    AddVariable,
    ModifyVariable,
    DeleteVariable,
}

export interface Task {
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

class ModifyVariableTask implements Task {
    t: TaskType;
    variable: string;
    values: string[];

    constructor(t: TaskType, variable: string, values: string[]) {
        this.t = t;
        this.variable = variable;
        this.values = values;
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

function createTask(t: TaskType, variable: string, value?: string, oldValue?: string, values?: string[]): Task {
    switch (t) {
        case TaskType.AddValue:
            return new AddValueTask(t, variable, value!);
        case TaskType.ModifyValue:
            return new ModifyValueTask(t, variable, oldValue!, value!);
        case TaskType.DeleteValue:
            return new DeleteValueTask(t, variable, value!);
        case TaskType.AddVariable:
            return new AddVariableTask(t, variable);
        case TaskType.ModifyVariable:
            return new ModifyVariableTask(t, variable, values!);
        case TaskType.DeleteVariable:
            return new DeleteVariableTask(t, variable);
    }
}

function preResolve(envs: EnvHashMap, task: Task) {
    switch (task.t) {
        case TaskType.AddValue:
            envs[task.variable].push((task as AddValueTask).value);
            break;
        case TaskType.ModifyValue: {
            const _task = task as ModifyValueTask;
            const index = envs[_task.variable].indexOf(_task.oldValue);
            envs[_task.variable].splice(index, 1, _task.newValue);
            break;
        }
        case TaskType.DeleteValue: {
            const _task = task as DeleteValueTask;
            const index1 = envs[_task.variable].indexOf(_task.value);
            envs[_task.variable].splice(index1, 1);
            break;
        }
        case TaskType.AddVariable:
            envs[task.variable] = [];
            break;
        case TaskType.ModifyVariable: {
            const _task = task as ModifyVariableTask;
            envs[task.variable] = _task.values;
            break;
        }
        case TaskType.DeleteVariable:
            delete envs[task.variable];
            break;
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

    optimise() {
        let _tasks: Task[] = [];

        while (this.queue.length > 0) {
            const task = this.queue.shift();
            // 向前寻找
            if (task) {
                switch (task.t) {
                    case TaskType.AddValue: {
                        const thisTask = task as AddValueTask;
                        // 向前寻找是否已经添加过值
                        const addValueTask = _tasks.find((t) =>
                            t.t === TaskType.AddValue
                            && t.variable === thisTask.variable
                            && (t as AddValueTask).value === thisTask.value
                        ) as AddValueTask;
                        if (addValueTask) {
                            // 如果找到了，删除旧的任务
                            _tasks = _tasks.filter((t) => t !== addValueTask);
                        }
                        _tasks.push(thisTask);
                        break;
                    }
                    case TaskType.ModifyValue: {
                        const thisTask = task as ModifyValueTask;
                        // 向前寻找add和modify操作
                        const addValueTask = _tasks.find((t) =>
                            t.t === TaskType.AddValue
                            && t.variable === thisTask.variable
                            && (t as AddValueTask).value === thisTask.oldValue
                        ) as AddValueTask;
                        if (addValueTask) {
                            addValueTask.value = thisTask.newValue;
                            break;
                        }

                        const modifyValueTask = _tasks.find((t) =>
                            t.t === TaskType.ModifyValue
                            && t.variable === thisTask.variable
                            && (t as ModifyValueTask).newValue === thisTask.oldValue
                        ) as ModifyValueTask;
                        if (modifyValueTask) {
                            modifyValueTask.newValue = thisTask.newValue;
                            break;
                        }

                        _tasks.push(thisTask);
                        break;
                    }
                    case TaskType.DeleteValue: {
                        const thisTask = task as DeleteValueTask;
                        // 寻找全部add和modify操作，并消除
                        const findAdd = (t: Task) => t.t === TaskType.AddValue
                            && t.variable === thisTask.variable
                            && (t as AddValueTask).value === thisTask.value
                        const ifAddValue = _tasks.find(findAdd);
                        _tasks = _tasks.filter(findAdd);

                        const findModify = (t: Task) => t.t === TaskType.ModifyValue
                            && t.variable === thisTask.variable
                            && (t as ModifyValueTask).newValue === thisTask.value
                        const ifModifyValue = _tasks.find(findModify);
                        _tasks = _tasks.filter(findModify);

                        if (ifAddValue || ifModifyValue) {
                            break;
                        } else {
                            _tasks.push(thisTask);
                        }
                        break;
                    }

                    case TaskType.AddVariable: {
                        const thisTask = task as AddVariableTask;
                        _tasks.push(thisTask);
                        break;
                    }

                    case TaskType.ModifyVariable: {
                        const thisTask = task as ModifyVariableTask;
                        _tasks.push(thisTask);
                        break;
                    }

                    case TaskType.DeleteVariable: {
                        const thisTask = task as DeleteVariableTask;
                        // 寻找全部add操作，并消除
                        const findAdd = (t: Task) => t.t === TaskType.AddVariable
                            && t.variable === thisTask.variable
                        const ifAddVariable = _tasks.find(findAdd);
                        _tasks = _tasks.filter(findAdd);

                        if (ifAddVariable) {
                            // 寻找所有value操作，并消除
                            const find = (t: Task) => (t.t === TaskType.AddValue
                                || t.t === TaskType.ModifyValue
                                || t.t === TaskType.DeleteValue)
                                && t.variable !== thisTask.variable
                            _tasks = _tasks.filter(find);
                            break;
                        }

                        _tasks.push(thisTask);
                        break;
                    }
                }
            }
        }

        this.queue = _tasks;
    }

    async execute() {
        const tempEnvMap = await get_all();
        
        while (this.queue.length > 0) {
            const task = this.queue.shift();
            // 向前寻找
            if (task) {
                switch (task.t) {
                    case TaskType.AddValue: {
                        const thisTask = task as AddValueTask;
                        const values = tempEnvMap[thisTask.variable];
                        await set_one(thisTask.variable, [...values, thisTask.value]);
                        break;
                    }
                    case TaskType.ModifyValue: {
                        const thisTask = task as ModifyValueTask;
                        const values = tempEnvMap[thisTask.variable];
                        const index = values.indexOf(thisTask.oldValue);
                        const result = values.splice(index, 1, thisTask.newValue);
                        await set_one(thisTask.variable, result);
                        break;
                    }
                    case TaskType.DeleteValue: {
                        const thisTask = task as DeleteValueTask;
                        const values = tempEnvMap[thisTask.variable];
                        const index = values.indexOf(thisTask.value);
                        const result = values.splice(index, 1);
                        await set_one(thisTask.variable, result);
                        break;
                    }

                    case TaskType.AddVariable: {
                        const thisTask = task as AddVariableTask;
                        await set_one(thisTask.variable, []);
                        break;
                    }

                    case TaskType.ModifyVariable: {
                        const thisTask = task as ModifyVariableTask;
                        await set_one(thisTask.variable, thisTask.values);
                        break;
                    }

                    case TaskType.DeleteVariable: {
                        const thisTask = task as DeleteVariableTask;
                        // await invoke("delete_one", { var: thisTask.variable });
                        console.error("[not support delete]", thisTask);
                        break;
                    }
                }
            }
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
    load: () => void;

    queue: TaskQueue;
    createTask: (t: TaskType, variable: string, value?: string, oldValue?: string) => Task;
    execute: () => Promise<void>;
}

const useEnvStore = create<IStore>((set, get) => ({
    envs: {},
    load: async () => {
        const envs = await get_all();
        set({ envs });
    },
    queue: new TaskQueue(),
    createTask: (t: TaskType, variable: string, value?: string, oldValue?: string) => {
        const task = createTask(t, variable, value, oldValue);
        set((state) => {
            state.queue.push(task);
            preResolve(state.envs, task);
            return state;
        })
        console.log(get().queue);
        return task;
    },
    execute: async () => {
        await get().queue.execute();
        const envs = await get_all();
        set({ envs });
    }
}));

export { useEnvStore as useEnvStore };

async function get_all(): Promise<EnvHashMap> {
    return invoke("get_all");
}

// async function get_one(variable: string): Promise<string[]> {
//     return invoke("get_one", { var: variable });
// }

async function set_one(variable: string, values: string[]): Promise<string[]> {
    const value = values.join(";");
    return invoke("set_one", { var: variable, value });
}


