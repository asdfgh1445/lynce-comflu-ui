mod config;
mod install;
mod models;
mod procs;

use procs::{log_tail, ProcMgr};
use serde::Serialize;
use std::fs;
use tauri::Manager;
use std::path::PathBuf;
use std::process::Command;

#[derive(Default)]
struct AppState {
    procs: ProcMgr,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppInfo {
    id: String,
    name: String,
    running: bool,
    pid: Option<u32>,
    port: u16,
    url: String,
    dir: String,
    dir_valid: bool,
    hint: String,
    port_listening: bool,
}

/// 探测端口是否已有服务在监听(用于识别终端手动启动的外部实例)
fn port_open(port: u16) -> bool {
    std::net::TcpStream::connect_timeout(
        &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        std::time::Duration::from_millis(300),
    )
    .is_ok()
}

fn sh(cmd: &str) -> String {
    Command::new("bash")
        .arg("-lc")
        .arg(cmd)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

#[tauri::command]
fn get_config() -> config::AppConfig {
    config::load()
}

#[tauri::command]
fn save_config(cfg: config::AppConfig) -> Result<(), String> {
    config::save(&cfg)
}

#[tauri::command]
fn get_apps(state: tauri::State<AppState>) -> Vec<AppInfo> {
    let cfg = config::load();
    let mut out = Vec::new();

    let sdir = PathBuf::from(config::expand_tilde(&cfg.swarm_dir));
    let swarm_valid = sdir.join("launch-macos.sh").exists();
    let (sr, sp) = state.procs.snapshot("swarm").unwrap_or((false, None));
    let swarm_listening = sr || port_open(cfg.swarm_port);
    out.push(AppInfo {
        id: "swarm".into(),
        name: "SwarmUI".into(),
        running: sr,
        pid: sp,
        port: cfg.swarm_port,
        url: format!("http://127.0.0.1:{}", cfg.swarm_port),
        dir: cfg.swarm_dir.clone(),
        dir_valid: swarm_valid,
        port_listening: swarm_listening,
        hint: if swarm_valid {
            if swarm_listening && !sr {
                "检测到端口已有 SwarmUI 在监听(可能是终端手动启动的实例)".into()
            } else {
                String::new()
            }
        } else {
            "未找到 launch-macos.sh,请在「高级选项」确认目录".into()
        },
    });

    let cdir = PathBuf::from(config::expand_tilde(&cfg.comfy_dir));
    let comfy_installed = cdir.join("main.py").exists();
    let venv_ok = cdir.join("venv/bin/python").exists();
    let (cr, cp) = state.procs.snapshot("comfy").unwrap_or((false, None));
    let comfy_listening = cr || port_open(cfg.comfy_port);
    out.push(AppInfo {
        id: "comfy".into(),
        name: "ComfyUI".into(),
        running: cr,
        pid: cp,
        port: cfg.comfy_port,
        url: format!("http://127.0.0.1:{}", cfg.comfy_port),
        dir: cfg.comfy_dir.clone(),
        dir_valid: comfy_installed && venv_ok,
        port_listening: comfy_listening,
        hint: if !comfy_installed {
            "尚未安装 —— 到「环境与安装」页一键安装".into()
        } else if !venv_ok {
            "缺少 venv 虚拟环境,请到「环境与安装」重新运行安装".into()
        } else if comfy_listening && !cr {
            "检测到端口已有 ComfyUI 在监听(可能是终端手动启动的实例)".into()
        } else {
            String::new()
        },
    });
    out
}

#[tauri::command]
fn start_app(state: tauri::State<AppState>, id: String) -> Result<(), String> {
    let cfg = config::load();
    let log = config::logs_dir().join(format!("{id}.log"));
    let mut cmd = Command::new("bash");
    match id.as_str() {
        "swarm" => {
            let dir = config::expand_tilde(&cfg.swarm_dir);
            if !PathBuf::from(&dir).join("launch-macos.sh").exists() {
                return Err("SwarmUI 目录无效(未找到 launch-macos.sh)".into());
            }
            // GUI 应用不继承登录 shell 的 PATH,dotnet 需要显式加入
            let script = format!(
                "export PATH=\"$HOME/dotnet:$PATH\"\ncd {} || exit 1\nexec ./launch-macos.sh {}\n",
                config::sh_quote(&dir),
                cfg.swarm_args.trim()
            );
            cmd.arg("-lc").arg(script).current_dir(&dir);
        }
        "comfy" => {
            let dir = config::expand_tilde(&cfg.comfy_dir);
            if !PathBuf::from(&dir).join("venv/bin/python").exists() {
                return Err("ComfyUI 未安装或缺少 venv,请先到「环境与安装」安装".into());
            }
            let script = format!(
                "export PYTORCH_ENABLE_MPS_FALLBACK=1\ncd {} || exit 1\nexec ./venv/bin/python main.py --port {} {}\n",
                config::sh_quote(&dir),
                cfg.comfy_port,
                cfg.comfy_args.trim()
            );
            cmd.arg("-lc").arg(script).current_dir(&dir);
        }
        _ => return Err(format!("未知应用: {id}")),
    }
    for line in cfg.extra_env.lines() {
        if let Some((k, v)) = line.split_once('=') {
            if !k.trim().is_empty() {
                cmd.env(k.trim(), v.trim());
            }
        }
    }
    state.procs.spawn(&id, cmd, &log)
}

#[tauri::command]
fn stop_app(state: tauri::State<AppState>, id: String) -> Result<(), String> {
    state.procs.stop(&id)
}

#[tauri::command]
fn get_logs(state: tauri::State<AppState>, id: String, bytes: Option<u32>) -> serde_json::Value {
    let b = bytes.unwrap_or(12000) as usize;
    let (running, pid) = state.procs.snapshot(&id).unwrap_or((false, None));
    let path = config::logs_dir().join(format!("{id}.log"));
    serde_json::json!({ "running": running, "pid": pid, "text": log_tail(&path, b) })
}

#[tauri::command]
fn list_models() -> Vec<models::ModelGroup> {
    models::scan(&config::load())
}

#[tauri::command]
fn install_comfy(state: tauri::State<AppState>) -> Result<(), String> {
    if state.procs.is_running(install::INSTALL_ID) {
        return Err("安装任务正在进行中".into());
    }
    install::start(&state.procs, &config::load())
}

#[tauri::command]
fn get_task(state: tauri::State<AppState>, id: String) -> serde_json::Value {
    let (running, pid) = state.procs.snapshot(&id).unwrap_or((false, None));
    let path = config::logs_dir().join(format!("{id}.log"));
    serde_json::json!({ "running": running, "pid": pid, "text": log_tail(&path, 20000) })
}

#[tauri::command]
fn open_url(url: String) -> Result<(), String> {
    Command::new("bash")
        .arg("-lc")
        .arg(format!("open {}", config::sh_quote(&url)))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn reveal_path(path: String) -> Result<(), String> {
    Command::new("bash")
        .arg("-lc")
        .arg(format!("open -R {}", config::sh_quote(&path)))
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
fn sys_info() -> serde_json::Value {
    let cfg = config::load();
    let py = config::sh_quote(&cfg.python_bin);
    let pyver = sh(&format!("{py} --version 2>&1"));
    let cdir = PathBuf::from(config::expand_tilde(&cfg.comfy_dir));
    let comfy_installed = cdir.join("main.py").exists() && cdir.join("venv/bin/python").exists();
    let sdir = PathBuf::from(config::expand_tilde(&cfg.swarm_dir));
    let df = sh("df -g \"$HOME\" | tail -1 | awk '{print $4}'");
    let free_gb: f64 = df.trim().parse().unwrap_or(0.0);
    serde_json::json!({
        "pythonVersion": pyver,
        "comfyInstalled": comfy_installed,
        "swarmValid": sdir.join("launch-macos.sh").exists(),
        "freeGb": free_gb,
    })
}

#[tauri::command]
fn upgrade_app(state: tauri::State<AppState>, id: String) -> Result<(), String> {
    let cfg = config::load();
    let task = format!("upgrade-{id}");
    if state.procs.is_running(&task) {
        return Err("升级任务正在进行中".into());
    }
    if state.procs.is_running(&id) {
        return Err("请先停止内核再升级".into());
    }
    let dir = match id.as_str() {
        "swarm" => config::expand_tilde(&cfg.swarm_dir),
        "comfy" => config::expand_tilde(&cfg.comfy_dir),
        _ => return Err(format!("未知应用: {id}")),
    };
    if !PathBuf::from(&dir).join(".git").exists() {
        return Err("该目录不是 git 仓库,无法升级".into());
    }
    let script = format!(
        "cd {} || exit 1\n\
         echo \"升级前: $(git log -1 --format='%h %s')\"\n\
         if git pull --ff-only 2>&1; then echo UPGRADE_DONE; else echo UPGRADE_FAILED; fi\n\
         echo \"升级后: $(git log -1 --format='%h %s')\"\n",
        config::sh_quote(&dir)
    );
    let log = config::logs_dir().join(format!("{task}.log"));
    let mut cmd = Command::new("bash");
    cmd.arg("-lc").arg(script).current_dir(&dir);
    state.procs.spawn(&task, cmd, &log)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionInfo {
    name: String,
    path: String,
    is_git: bool,
}

#[tauri::command]
fn list_extensions() -> Vec<ExtensionInfo> {
    let cfg = config::load();
    let mut out = Vec::new();
    let cn = PathBuf::from(config::expand_tilde(&cfg.comfy_dir)).join("custom_nodes");
    if let Ok(rd) = fs::read_dir(&cn) {
        for entry in rd.flatten() {
            let p = entry.path();
            if !p.is_dir() {
                continue;
            }
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') || name == "__pycache__" {
                continue;
            }
            out.push(ExtensionInfo {
                name,
                path: p.to_string_lossy().into_owned(),
                is_git: p.join(".git").exists(),
            });
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    out
}

#[tauri::command]
fn install_extension(state: tauri::State<AppState>, url: String) -> Result<(), String> {
    let cfg = config::load();
    if state.procs.is_running("ext-install") {
        return Err("扩展安装正在进行中".into());
    }
    let u = url.trim().trim_end_matches('/').to_string();
    if !u.starts_with("https://") && !u.starts_with("http://") {
        return Err("请输入 http(s) 开头的 git 仓库地址".into());
    }
    let name = u
        .rsplit('/')
        .next()
        .unwrap_or("")
        .trim_end_matches(".git")
        .to_string();
    if name.is_empty() {
        return Err("无法从 URL 解析扩展名".into());
    }
    let cn = PathBuf::from(config::expand_tilde(&cfg.comfy_dir)).join("custom_nodes");
    if !cn.exists() {
        return Err("ComfyUI 未安装,请先到「环境与安装」安装".into());
    }
    let dest = cn.join(&name);
    if dest.exists() {
        return Err(format!("扩展 {name} 已存在"));
    }
    let script = format!(
        "git clone --depth 1 {u} {d} 2>&1 \
         || git clone --depth 1 https://gh-proxy.com/{u} {d} 2>&1 \
         || git clone --depth 1 https://ghfast.top/{u} {d} 2>&1\n\
         if [ -d {d}/.git ]; then echo EXT_DONE; else echo EXT_FAILED; fi\n",
        u = config::sh_quote(&u),
        d = config::sh_quote(&dest.to_string_lossy())
    );
    let log = config::logs_dir().join("ext-install.log");
    let mut cmd = Command::new("bash");
    cmd.arg("-lc").arg(script).current_dir(&cn);
    state.procs.spawn("ext-install", cmd, &log)
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_config,
            save_config,
            get_apps,
            start_app,
            stop_app,
            get_logs,
            list_models,
            install_comfy,
            get_task,
            open_url,
            reveal_path,
            sys_info,
            upgrade_app,
            list_extensions,
            install_extension
        ])
        .build(tauri::generate_context!())
        .expect("构建 Tauri 应用失败")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                app.state::<AppState>().procs.stop_all();
            }
        });
}
