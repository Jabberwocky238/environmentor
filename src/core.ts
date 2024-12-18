import { invoke } from "@tauri-apps/api/core";
import { create } from "zustand";

// function preResolve(envs: EnvHashMap, task: Task) {
//     switch (task.t) {
//         case TaskType.AddValue:
//             envs[task.variable].push((task as AddValueTask).value);
//             break;
//         case TaskType.ModifyValue: {
//             const _task = task as ModifyValueTask;
//             const index = envs[_task.variable].indexOf(_task.oldValue);
//             envs[_task.variable].splice(index, 1, _task.newValue);
//             break;
//         }
//         case TaskType.DeleteValue: {
//             const _task = task as DeleteValueTask;
//             const index1 = envs[_task.variable].indexOf(_task.value);
//             envs[_task.variable].splice(index1, 1);
//             break;
//         }
//         case TaskType.AddVariable:
//             envs[task.variable] = [];
//             break;
//         case TaskType.ModifyVariable: {
//             const _task = task as ModifyVariableTask;
//             envs[task.variable] = _task.values;
//             break;
//         }
//         case TaskType.DeleteVariable:
//             delete envs[task.variable];
//             break;
//     }
// }
// export class TaskQueue {
//     queue: Task[] = []

//     push(task: Task) {
//         this.queue.push(task);
//     }

//     remove(task: Task) {
//         const index = this.queue.indexOf(task);
//         if (index > -1) {
//             this.queue.splice(index, 1);
//         }
//     }

//     async execute() {
//         const tempEnvMap = await get_all();
        
//         while (this.queue.length > 0) {
//             const task = this.queue.shift();
//             // 向前寻找
//             if (task) {
//                 switch (task.t) {
//                     case TaskType.AddValue: {
//                         const thisTask = task as AddValueTask;
//                         const values = tempEnvMap[thisTask.variable];
//                         await set_one(thisTask.variable, [...values, thisTask.value]);
//                         break;
//                     }
//                     case TaskType.ModifyValue: {
//                         const thisTask = task as ModifyValueTask;
//                         const values = tempEnvMap[thisTask.variable];
//                         const index = values.indexOf(thisTask.oldValue);
//                         const result = values.splice(index, 1, thisTask.newValue);
//                         await set_one(thisTask.variable, result);
//                         break;
//                     }
//                     case TaskType.DeleteValue: {
//                         const thisTask = task as DeleteValueTask;
//                         const values = tempEnvMap[thisTask.variable];
//                         const index = values.indexOf(thisTask.value);
//                         const result = values.splice(index, 1);
//                         await set_one(thisTask.variable, result);
//                         break;
//                     }

//                     case TaskType.AddVariable: {
//                         const thisTask = task as AddVariableTask;
//                         await set_one(thisTask.variable, []);
//                         break;
//                     }

//                     case TaskType.ModifyVariable: {
//                         const thisTask = task as ModifyVariableTask;
//                         await set_one(thisTask.variable, thisTask.values);
//                         break;
//                     }

//                     case TaskType.DeleteVariable: {
//                         const thisTask = task as DeleteVariableTask;
//                         // await invoke("delete_one", { var: thisTask.variable });
//                         console.error("[not support delete]", thisTask);
//                         break;
//                     }
//                 }
//             }
//         }
//     }

//     clear() {
//         this.queue = [];
//     }

//     size(): number {
//         return this.queue.length;
//     }

//     isEmpty(): boolean {
//         return this.queue.length === 0;
//     }
// }

export type EnvHashMap = { [key: string]: string[] };
export type AppState = {
    env: EnvHashMap,
    task_queue: {
        tasks: Task[],
    }
}

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

// load, get_state, add_task

async function load(): Promise<void> {
    return invoke("load");
}
async function get_state(): Promise<EnvHashMap> {
    return invoke("get_state");
}
// async function get_one(variable: string): Promise<string[]> {
//     return invoke("get_one", { var: variable });
// }

async function set_one(variable: string, values: string[]): Promise<string[]> {
    const value = values.join(";");
    return invoke("set_one", { var: variable, value });
}


