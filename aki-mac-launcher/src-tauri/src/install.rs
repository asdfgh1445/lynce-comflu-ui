use crate::config::{self, AppConfig};
use crate::procs::ProcMgr;
use std::path::PathBuf;
use std::process::Command;

pub const INSTALL_ID: &str = "comfy-install";

/// ComfyUI 一键安装脚本:clone 内核 → venv → PyTorch(MPS) → 依赖。
/// 镜像策略对齐绘世 data.json 的 PyPI 优先级链(豆瓣→腾讯云→阿里云→清华)。
pub fn script(cfg: &AppConfig) -> String {
    let dir = config::sh_quote(&config::expand_tilde(&cfg.comfy_dir));
    let py = config::sh_quote(&cfg.python_bin);
    format!(
        r#"set -e
echo "== AkiHuishi ComfyUI 一键安装 (Apple Silicon / MPS) =="
echo "目标目录: {dir}   Python: {py}"
mkdir -p "$(dirname {dir})"
if [ ! -d "{dir}/.git" ]; then
  rm -rf {dir}
  git clone --depth 1 https://github.com/comfyanonymous/ComfyUI {dir} \
    || git clone --depth 1 https://gh-proxy.com/https://github.com/comfyanonymous/ComfyUI {dir} \
    || git clone --depth 1 https://ghfast.top/https://github.com/comfyanonymous/ComfyUI {dir}
fi
cd {dir}
[ -x venv/bin/python ] || {py} -m venv venv
./venv/bin/python -m pip install --upgrade pip || true

MIRRORS="https://pypi.doubanio.com/simple https://mirrors.cloud.tencent.com/pypi/simple https://mirrors.aliyun.com/pypi/simple https://pypi.tuna.tsinghua.edu.cn/simple"
install_pkgs() {{
  for m in $MIRRORS; do
    echo "[mirror] 使用 $m"
    ./venv/bin/python -m pip install $1 -i "$m" && return 0
  done
  echo "[mirror] 全部镜像失败,回退官方源"
  ./venv/bin/python -m pip install $1 && return 0
  return 1
}}

install_pkgs "torch torchvision"
install_pkgs "-r requirements.txt"
echo "=== INSTALL_OK ==="
"#
    )
}

pub fn start(state: &ProcMgr, cfg: &AppConfig) -> Result<(), String> {
    let dir = PathBuf::from(config::expand_tilde(&cfg.comfy_dir));
    let cwd = dir
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(std::env::temp_dir);
    std::fs::create_dir_all(&cwd).map_err(|e| e.to_string())?;
    let log = config::logs_dir().join("comfy-install.log");
    let mut cmd = Command::new("bash");
    cmd.arg("-lc").arg(script(cfg)).current_dir(&cwd);
    state.spawn(INSTALL_ID, cmd, &log)
}
