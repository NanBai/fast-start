import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";

// 桌面工具不暴露 WebView 默认右键菜单，避免用户看到 Reload / Inspect Element。
document.addEventListener("contextmenu", (event) => {
  event.preventDefault();
});

ReactDOM.createRoot(document.getElementById("root") as HTMLElement).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
