import { useEffect, useState } from "react";
import { api, AppInfo, AppConfig } from "./api";
import StartTab from "./tabs/StartTab";
import InstallTab from "./tabs/InstallTab";
import ModelsTab from "./tabs/ModelsTab";
import ExtensionsTab from "./tabs/ExtensionsTab";
import SettingsTab from "./tabs/SettingsTab";
import logo from "./assets/logo.png";

const TABS = [
  { id: "start", label: "一键启动", icon: "▶" },
  { id: "install", label: "环境与安装", icon: "⬇" },
  { id: "models", label: "模型管理", icon: "🗂" },
  { id: "ext", label: "扩展管理", icon: "🧩" },
  { id: "settings", label: "高级选项", icon: "⚙" },
  { id: "about", label: "关于", icon: "ℹ" },
] as const;

type TabId = (typeof TABS)[number]["id"];

export default function App() {
  const [tab, setTab] = useState<TabId>("start");
  const [apps, setApps] = useState<AppInfo[]>([]);
  const [cfg, setCfg] = useState<AppConfig | null>(null);

  const refresh = () => api.getApps().then(setApps).catch(() => {});
  useEffect(() => {
    refresh();
    api.getConfig().then(setCfg).catch(() => {});
    const t = setInterval(refresh, 2000);
    return () => clearInterval(t);
  }, []);

  const running = apps.some((a) => a.running);

  return (
    <div className="layout">
      <aside className="sidebar">
        <div className="brand">
          <img className="logo-img" src={logo} alt="Lynce" />
          <div>
            <div className="brand-name">Lynce AI 启动器</div>
            <div className="brand-sub">v0.2 · for macOS</div>
          </div>
        </div>
        {TABS.map((t) => (
          <button
            key={t.id}
            className={"nav" + (tab === t.id ? " active" : "")}
            onClick={() => setTab(t.id)}
          >
            <span className="nav-icon">{t.icon}</span>
            {t.label}
          </button>
        ))}
        <div className="sidebar-foot">
          <span className={"dot " + (running ? "on" : "off")} />
          <span>{running ? "有任务运行中" : "空闲"}</span>
        </div>
      </aside>
      <main className="content">
        {tab === "start" && <StartTab apps={apps} onChanged={refresh} />}
        {tab === "install" && <InstallTab onChanged={refresh} />}
        {tab === "models" && <ModelsTab />}
        {tab === "ext" && <ExtensionsTab />}
        {tab === "settings" && cfg && (
          <SettingsTab
            cfg={cfg}
            onSaved={(c) => {
              setCfg(c);
              refresh();
            }}
          />
        )}
        {tab === "about" && <About />}
      </main>
    </div>
  );
}

function About() {
  return (
    <div className="card">
      <h2>Lynce AI 启动器</h2>
      <p>
        面向 Apple Silicon Mac 的本地 AI 绘图启动器,使用 Rust + React + Tauri
        构建,原生运行、无需任何运行时依赖。
      </p>
      <p>
        功能:SwarmUI / ComfyUI 一键启停(含外部实例检测)、ComfyUI
        一键安装(MPS 加速、多镜像回退)、内核升级、扩展管理、模型库扫描、自定义启动参数与环境变量注入。
      </p>
      <p className="muted">版本 v0.2.0 · 仅限本机个人使用</p>
    </div>
  );
}
