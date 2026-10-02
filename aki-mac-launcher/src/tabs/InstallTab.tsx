import { useEffect, useRef, useState } from "react";
import { api, SysInfo } from "../api";

export default function InstallTab({ onChanged }: { onChanged: () => void }) {
  const [sys, setSys] = useState<SysInfo | null>(null);
  const [log, setLog] = useState("");
  const [running, setRunning] = useState(false);
  const [err, setErr] = useState("");
  const ref = useRef<HTMLPreElement>(null);

  useEffect(() => {
    api.sysInfo().then(setSys).catch(() => {});
    let alive = true;
    const pull = () =>
      api
        .getTask("comfy-install")
        .then((r) => {
          if (!alive) return;
          setLog(r.text);
          setRunning(r.running);
        })
        .catch(() => {});
    pull();
    const t = setInterval(pull, 2000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, []);

  useEffect(() => {
    if (ref.current) ref.current.scrollTop = ref.current.scrollHeight;
    if (log.includes("INSTALL_OK")) onChanged();
  }, [log, onChanged]);

  const install = async () => {
    setErr("");
    try {
      await api.installComfy();
    } catch (e) {
      setErr(String(e));
    }
  };

  return (
    <div>
      <h1>环境与安装</h1>
      {err && <div className="error">{err}</div>}
      <div className="card">
        <h3>ComfyUI(官方内核 · Apple Silicon MPS 版)</h3>
        <p className="muted">
          自动 clone 内核 → 创建 Python venv → 安装 Apple Silicon 版 PyTorch(MPS
          加速)→ 安装依赖。含 GitHub / PyPI 国内镜像回退。
          {sys && (
            <>
              {" "}
              已安装: <b>{sys.comfyInstalled ? "是" : "否"}</b> · Python:{" "}
              <b>{sys.pythonVersion || "?"}</b> · 磁盘可用:{" "}
              <b>{sys.freeGb} GB</b>
            </>
          )}
        </p>
        <div className="row">
          <button className="btn primary" disabled={running} onClick={install}>
            {running ? "安装中…" : sys?.comfyInstalled ? "重新安装/修复" : "一键安装 ComfyUI"}
          </button>
          {running && <span className="muted">首次安装约需 10-30 分钟(含 PyTorch 下载)</span>}
        </div>
        {(running || log) && (
          <pre className="logbox" ref={ref}>
            {log || "(等待日志…)"}
          </pre>
        )}
      </div>
      <div className="card">
        <h3>SwarmUI</h3>
        <p className="muted">
          {sys?.swarmValid ? "已检测到安装(默认 ~/SwarmUI)。" : "未找到(默认 ~/SwarmUI)。"}
          SwarmUI 由官方安装脚本自行管理,本启动器只负责一键启停与 dotnet 路径注入,不重复安装。
        </p>
      </div>
    </div>
  );
}
