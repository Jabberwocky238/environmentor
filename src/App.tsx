
import Main from "@/views/Main";
import Setting from "@/views/Setting";
import History from "@/views/History";

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

function App() {
  const routes = [
    { href: "/", title: 'Main' },
    { href: "/History", title: 'History' },
    { href: "/setting", title: 'Setting' },
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
            <Main />
          </Route>
          <Route path="/History">
            <History />
          </Route>
          <Route path="/setting">
            <Setting />
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
