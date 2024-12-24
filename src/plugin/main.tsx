import React from "react";
import ReactDOM from "react-dom/client";
import App from "./Index";

import "@/styles/main.scss";

const StrictApp = () => (
  <React.StrictMode>
    <App />
  </React.StrictMode>
);

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<StrictApp/>);
// ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<App/>);
