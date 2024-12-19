import { useEffect } from "react";
import { useEnv } from "@/core";

import EnvList from "@@/main/EnvList";
import ValueList from "@@/main/ValueList";
import Control from "@@/main/Control";

export default function Main() {
    const store = useEnv();

    useEffect(() => {
        store.load();
    }, []);

    return (
        <div className="row">
            <div className="var-col">
                <EnvList></EnvList>
            </div>
            <div className="main-col">
                <Control></Control>
                <ValueList></ValueList>
            </div>
        </div>
    )
}