import { useEffect, useRef, useState } from "react";
import { api, ExtensionInfo } from "../api";

export default function ExtensionsTab() {
  const [exts, setExts] = useState<ExtensionInfo[]>([]);
  const [url, setUrl] = useState("");
  const [msg, setMsg] = useState("");
  const [log, setLog] = useState("");
  const [installing, setInstalling] = useState(false);
  const ref = useRef<HTMLPreElement>(null);
  const wasRunning = useRef(false);

  const reload = () => api.listExtensions().then(setExts).catch(() => {});
  useEffect(() => {
    reload();
    const t = setInterval(async () => {
      const r = await api.getTask("ext-install").catch(() => null);
      if (r) {
        setLog(r.text);
        setInstalling(r.running);
        if (wasRunning.current && !r.running) reload();
        wasRunning.current = r.running;
      }
    }, 2000);
    return () => clearInterval(t);
  }, []);
  useEffect(() => {
    if (ref.current) ref.current.scrollTop = ref.current.scrollHeight;
  }, [log]);

  const install = async () => {
    setMsg("");
    if (!url.trim()) {
      setMsg("请输入扩展的 git 仓库地址");
      return;
    }
    try {
      await api.installExtension(url.trim());
    } catch (e) {
      setMsg(String(e));
    }
  };

  return (
    <div>
      <h1>扩展管理</h1>
      <p className="muted">
        管理 ComfyUI 的 custom_nodes 扩展(SwarmUI 的扩展机制不同,请在其界面内管理)。
        当前 {exts.length} 个扩展。
      </p>
      <div className="card">
        <h3>从 git 安装新扩展</h3>
        <div className="row">
          <input
            className="grow"
            style={{ marginTop: 0 }}
            placeholder="https://github.com/xxx/ComfyUI-SomeExtension"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
          />
          <button className="btn primary" disabled={installing} onClick={install}>
            {installing ? "安装中…" : "安装"}
          </button>
        </div>
        {msg && <div className="hint">{msg}</div>}
        {(installing || log) && (
          <pre className="logbox" ref={ref}>
            {log || "(等待日志…)"}
          </pre>
        )}
      </div>
      <div className="card">
        {exts.length === 0 && <p className="muted">尚未安装任何扩展(或 ComfyUI 未安装)。</p>}
        {exts.map((e) => (
          <div key={e.path} className="mitem">
            <span>
              {e.name}
              {e.isGit ? "" : <span className="muted">(非 git 目录)</span>}
            </span>
            <button className="btn ghost" onClick={() => api.revealPath(e.path)}>
              Finder
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
