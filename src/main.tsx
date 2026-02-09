import React from "react";
import ReactDOM from "react-dom/client";
import { HashRouter, Routes, Route } from "react-router-dom";
import App from "./App";
import Settings from "./Settings";
import Manager from "./manager/Manager";
import "./App.css";

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <HashRouter>
      <Routes>
        <Route path="/" element={<App />} />
        <Route path="/settings" element={<Settings />} />
        <Route path="/manager" element={<Manager />} />
      </Routes>
    </HashRouter>
  </React.StrictMode>,
);
