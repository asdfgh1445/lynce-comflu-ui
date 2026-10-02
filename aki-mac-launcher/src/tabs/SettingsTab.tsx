import { useState } from "react";
import { api, AppConfig } from "../api";

export default function SettingsTab({
  cfg,
  onSaved,
}: {
  cfg: AppConfig;
  onSaved: (c: AppConfig) => void;
}) {
  const [form, setForm] = useState<AppConfig>(cfg);
  const [msg, setMsg] = useState("");
  const set = (k: keyof AppConfig, v: unknown) => setForm({ ...form, [k]: v });

  const save = async () => {
    try {
      await api.saveConfig(form);
      setMsg("已保存 ✔");
      onSaved(form);
      setTimeout(() => setMsg(""), 2000);
    } catch (e) {
      setMsg(String(e));
    }
  };

  return (
    <div>
      <h1>高级选项</h1>
      <div className="card">
        <label>
          SwarmUI 目录
          <input value={form.swarmDir} onChange={(e) => set("swarmDir", e.target.value)} />
        </label>
        <label>
          SwarmUI 端口
          <input
            type="number"
            value={form.swarmPort}
            onChange={(e) => set("swarmPort", +e.target.value)}
          />
        </label>
        <label>
          ComfyUI 目录
          <input value={form.comfyDir} onChange={(e) => set("comfyDir", e.target.value)} />
        </label>
        <label>
          ComfyUI 端口
          <input
            type="number"
            value={form.comfyPort}
            onChange={(e) => set("comfyPort", +e.target.value)}
          />
        </label>
        <label>
          Python 解释器(用于创建 venv)
          <input value={form.pythonBin} onChange={(e) => set("pythonBin", e.target.value)} />
        </label>
        <label>
          SwarmUI 自定义启动参数(原样拼接,如 --listen 0.0.0.0)
          <input value={form.swarmArgs} onChange={(e) => set("swarmArgs", e.target.value)} />
        </label>
        <label>
          ComfyUI 自定义启动参数(如 --lowvram --use-pytorch-cross-attention)
          <input value={form.comfyArgs} onChange={(e) => set("comfyArgs", e.target.value)} />
        </label>
        <label className="checkline">
          <input
            type="checkbox"
            checked={form.autoOpen}
            onChange={(e) => set("autoOpen", e.target.checked)}
          />
          启动成功后自动打开浏览器
        </label>
        <label>
          额外模型目录(每行一个)
          <textarea
            rows={3}
            value={form.extraModelDirs.join("\n")}
            onChange={(e) =>
              set(
                "extraModelDirs",
                e.target.value.split("\n").map((s) => s.trim()).filter(Boolean)
              )
            }
          />
        </label>
        <label>
          额外环境变量(每行 KEY=VALUE,启动内核时注入)
          <textarea rows={3} value={form.extraEnv} onChange={(e) => set("extraEnv", e.target.value)} />
        </label>
        <div className="row">
          <button className="btn primary" onClick={save}>
            保存配置
          </button>
          {msg && <span className="muted">{msg}</span>}
        </div>
      </div>
    </div>
  );
}
