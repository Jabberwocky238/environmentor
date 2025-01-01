import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

import "./styles/root.scss";
import "./styles/nodefault.css";

const StrictApp = () => (
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<StrictApp/>);
// ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<App/>);
