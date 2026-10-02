import { useEffect, useRef, useState } from "react";
import { api, AppInfo } from "../api";

function LogBox({ id }: { id: string }) {
  const [text, setText] = useState("");
  const ref = useRef<HTMLPreElement>(null);
  useEffect(() => {
    let alive = true;
    const pull = () =>
      api
        .getLogs(id)
        .then((r) => {
          if (alive) setText(r.text);
        })
        .catch(() => {});
    pull();
    const t = setInterval(pull, 2000);
    return () => {
      alive = false;
      clearInterval(t);
    };
  }, [id]);
  useEffect(() => {
    if (ref.current) ref.current.scrollTop = ref.current.scrollHeight;
  }, [text]);
  return <pre className="logbox" ref={ref}>{text || "(暂无日志)"}</pre>;
}

export default function StartTab({
  apps,
  onChanged,
}: {
  apps: AppInfo[];
  onChanged: () => void;
}) {
  const [openLog, setOpenLog] = useState<string | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [err, setErr] = useState("");
  const [upgrading, setUpgrading] = useState<string | null>(null);
  const [upMsg, setUpMsg] = useState("");
  const autoOpenRef = useRef(true);

  useEffect(() => {
    api.getConfig().then((c) => (autoOpenRef.current = c.autoOpen)).catch(() => {});
  }, []);

  // 升级任务轮询:结束后取日志尾部作为结果提示
  useEffect(() => {
    if (!upgrading) return;
    const t = setInterval(async () => {
      const r = await api.getTask(`upgrade-${upgrading}`).catch(() => null);
      if (r && !r.running) {
        setUpgrading(null);
        const tail = r.text.trim().split("\n").filter(Boolean).slice(-2).join(" · ");
        setUpMsg(r.text.includes("UPGRADE_DONE") ? `升级完成 ✔ ${tail}` : `升级未完成:${tail}`);
        onChanged();
      }
    }, 1500);
    return () => clearInterval(t);
  }, [upgrading, onChanged]);

  const autoOpenWhenReady = async (id: string) => {
    if (!autoOpenRef.current) return;
    for (let i = 0; i < 60; i++) {
      await new Promise((r) => setTimeout(r, 1500));
      const list = await api.getApps().catch(() => []);
      const me = list.find((x) => x.id === id);
      if (!me) break;
      if (me.portListening) {
        api.openUrl(me.url);
        return;
      }
      if (!me.running) return;
    }
  };

  const doAct = async (id: string, act: "start" | "stop") => {
    setBusy(id + act);
    setErr("");
    try {
      await (act === "start" ? api.startApp(id) : api.stopApp(id));
      if (act === "start") await autoOpenWhenReady(id);
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(null);
      onChanged();
    }
  };

  const doUpgrade = async (id: string) => {
    setUpMsg("");
    try {
      await api.upgradeApp(id);
      setUpgrading(id);
    } catch (e) {
      setUpMsg(String(e));
    }
  };

  return (
    <div>
      <h1>一键启动</h1>
      {err && <div className="error">{err}</div>}
      {upMsg && <div className="hint">{upMsg}</div>}
      <div className="cards">
        {apps.map((a) => {
          const live = a.running || a.portListening;
          return (
            <div className="card appcard" key={a.id}>
              <div className="app-head">
                <div>
                  <div className="app-name">{a.name}</div>
                  <div className="app-dir muted">{a.dir}</div>
                </div>
                <span className={"status " + (live ? "run" : a.dirValid ? "stop" : "bad")}>
                  {a.running
                    ? "● 运行中"
                    : a.portListening
                      ? "● 运行中(外部实例)"
                      : a.dirValid
                        ? "○ 已停止"
                        : "✕ 未就绪"}
                </span>
              </div>
              {a.hint && <div className="hint">{a.hint}</div>}
              <div className="row">
                {a.running ? (
                  <button
                    className="btn danger"
                    disabled={busy === a.id + "stop"}
                    onClick={() => doAct(a.id, "stop")}
                  >
                    停止
                  </button>
                ) : (
                  <button
                    className="btn primary"
                    disabled={live || !a.dirValid || busy === a.id + "start"}
                    onClick={() => doAct(a.id, "start")}
                  >
                    启动
                  </button>
                )}
                <button className="btn" disabled={!live} onClick={() => api.openUrl(a.url)}>
                  打开界面
                </button>
                <button
                  className="btn ghost"
                  disabled={a.running || !!upgrading}
                  title="git pull 升级内核"
                  onClick={() => doUpgrade(a.id)}
                >
                  {upgrading === a.id ? "升级中…" : "升级"}
                </button>
                <button
                  className="btn ghost"
                  onClick={() => setOpenLog(openLog === a.id ? null : a.id)}
                >
                  {openLog === a.id ? "收起日志" : "查看日志"}
                </button>
              </div>
              {a.running && (
                <div className="muted small">
                  PID {a.pid} · {a.url}
                </div>
              )}
              {a.portListening && !a.running && (
                <div className="muted small">
                  外部实例 ·{" "}
                  <a className="link" onClick={() => api.openUrl(a.url)}>
                    {a.url}
                  </a>
                </div>
              )}
              {openLog === a.id && <LogBox id={a.id} />}
            </div>
          );
        })}
      </div>
    </div>
  );
}
