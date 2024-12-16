import { useState, useEffect } from "react";
// import reactLogo from "./assets/react.svg";
import { type EnvHashMap, useEnvStore } from "./core";
import "./App.css";
import "./custom.css";
import EnvList from "./components/EnvList";
import ValueList from "./components/ValueList";


function App() {
  const store = useEnvStore();

  useEffect(() => {
    store.get_all().then((envs) => {
      console.log(envs);
    });
  }, []);

  return (
    <main className="container">
      <div className="row">
        <div className="col30">
          <EnvList></EnvList>
        </div>

        <div className="col70">
          <ValueList></ValueList>

          <div className="console">


          </div>

        </div>
      </div>
    </main>
  );
}

export default App;
