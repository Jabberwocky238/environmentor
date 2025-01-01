
import Notification from "@@/utils/Notification";

import "./styles/App.scss";
import { Link, Route, Switch } from "wouter";
import { emitter } from "@/core";
import { useEffect } from "react";
import { event } from "@tauri-apps/api";


function backendEventResolver(n: { event: string, id: number, payload: any }) {
  switch (n.event) {
    case "notification":
      return emitter.emit("notification", n.payload);
    default:
      return n;
  }
}

import Main from "@/views/Main";
import Setting from "@/views/Setting";
import FSTree from "@/views/FSTree";

function App() {
  useEffect(() => {
    const unlisten = event.listen("notification", (n: any) => {
      console.log("[App useEffect] notification", n);
      backendEventResolver(n);
    })
    return () => {
      unlisten.then((u) => u());
    }
  }, []);

  const _active = (active: boolean) => (active ? "tab-item-active" : "");
  return (
    <>
      <div className="tab">
        <Link className={_active} href="/">Main</Link>
        <Link className={_active} href="/FSTree">FSTree</Link>
        <Link className={_active} href="/Setting">Setting</Link>
      </div>
      <div className="tab-content">
        <Switch>
          <Route path="/"><Main /></Route>
          <Route path="/FSTree"><FSTree /></Route>
          <Route path="/Setting"><Setting /></Route>
          {/* Default route in a switch */}
          <Route>
            <h1>404: No such page!</h1>
          </Route>
        </Switch>
      </div>
      <Notification />
    </>
  );
}

export default App;
