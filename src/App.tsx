
import Tab from "@@/Tab";
import Main from "@/views/Main";
import Record from "@/views/Record";

import "./App.css";
import "./App.scss";
import "./custom.scss";
import { useState } from "react";


function App() {
  const options = ["Main", "History"];
  const [selected, setSelected] = useState(options[0]);

  return (
    <main className="container">
      <Tab options={options} selected={selected} setSelected={(_selected) => {
        setSelected(_selected);
      }} ></Tab>
      <div className="tab-content">
        <Main style={{ display: selected === "Main" ? "flex" : 'none' }}></Main>
        <Record style={{ display: selected === "History" ? "flex" : 'none' }}></Record>
      </div>
    </main>
  );
}

export default App;
