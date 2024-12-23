
import Main from "@/views/Main";

import "./App.css";
import "./App.scss";
import "./custom.scss";
import { Link, Route, Switch } from "wouter";

function App() {
  const routes = [
    { href: "/", title: 'Main' },
    // { href: "/history", title: 'History' }
  ]

  return (
    <>
      {/* <div className="tab">
        {routes.map((r) => (
          <Link key={r.href} className={(active) => (active ? "tab-item-active" : "")} href={r.href}>{r.title}</Link>
        ))}
      </div> */}
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
    </>
  );
}

export default App;
