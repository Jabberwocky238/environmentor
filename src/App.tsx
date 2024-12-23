
import Main from "@/views/Main";
import Notification from "@@/utils/Notification";

import "./App.css";
import "./App.scss";
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

function App() {
  const routes = [
    { href: "/", title: 'Main' },
    // { href: "/history", title: 'History' }
  ]

  useEffect(() => {
    const unlisten = event.listen("notification", (n: any) => {
      console.log("[App useEffect] notification", n);
      backendEventResolver(n);
    })
    return () => {
      unlisten.then((u) => u());
    }
  }, []);

  return (
    <>
      <div className="tab">
        {routes.map((r) => (
          <Link key={r.href} className={(active) => (active ? "tab-item-active" : "")} href={r.href}>{r.title}</Link>
        ))}
      </div>
      <div className="tab-content">
        <Switch>
          <Route path="/">
            <Main></Main>
          </Route>
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
