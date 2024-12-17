import { useEffect } from "react";
// import reactLogo from "./assets/react.svg";
import { useEnvStore } from "./core";
import "./App.css";
import "./custom.scss";
import EnvList from "./components/EnvList";
import ValueList from "./components/ValueList";

function App() {
  const envStore = useEnvStore();

  useEffect(() => {
    envStore.load();
  }, []);

  return (
    <main className="container">
      <div className="row">
        <div className="var-col">
          <EnvList></EnvList>
        </div>
        <div className="main-col">
          <ValueList></ValueList>
        </div>
      </div>
    </main>
  );
}

export default App;
