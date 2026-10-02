import { useEffect, useState } from "react";
import { api, ModelGroup } from "../api";

function fmtSize(mb: number) {
  return mb >= 1024 ? (mb / 1024).toFixed(2) + " GB" : mb.toFixed(0) + " MB";
}

export default function ModelsTab() {
  const [groups, setGroups] = useState<ModelGroup[]>([]);
  const [open, setOpen] = useState<string | null>(null);
  useEffect(() => {
    api.listModels().then(setGroups).catch(() => {});
  }, []);

  const total = groups.reduce((s, g) => s + g.count, 0);
  const totalMb = groups.reduce((s, g) => s + g.totalMb, 0);

  return (
    <div>
      <h1>模型管理</h1>
      <p className="muted">
        扫描 SwarmUI Models 目录与自定义目录:共 {total} 个模型文件 · {fmtSize(totalMb)}
      </p>
      <div className="card">
        {groups.length === 0 && <p className="muted">未发现模型文件。</p>}
        {groups.map((g) => (
          <div key={g.dir} className="mgroup">
            <div className="mrow" onClick={() => setOpen(open === g.dir ? null : g.dir)}>
              <span className="mname">{g.label}</span>
              <span className="muted">
                {g.count} 个 · {fmtSize(g.totalMb)}
              </span>
              <span className="muted small grow">{g.dir}</span>
              <button
                className="btn ghost"
                onClick={(e) => {
                  e.stopPropagation();
                  api.revealPath(g.items[0]?.path || g.dir);
                }}
              >
                Finder
              </button>
            </div>
            {open === g.dir && (
              <div className="mlist">
                {g.items.map((i) => (
                  <div key={i.path} className="mitem">
                    <span>{i.name}</span>
                    <span className="muted">{fmtSize(i.sizeMb)}</span>
                  </div>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
