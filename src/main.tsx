import ReactDOM from "react-dom/client";
import App from "./App";

import "./styles/nodefault.css";
import "./styles/root.scss";

// import React from "react";
// ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
//   <React.StrictMode>
//     <App />
//   </React.StrictMode>
// );
ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(<App />);
