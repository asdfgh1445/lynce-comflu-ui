use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// macOS libSystem 自带,避免第三方依赖
extern "C" {
    fn kill(pid: i32, sig: i32) -> i32;
}

const SIGTERM: i32 = 15;
const SIGKILL: i32 = 9;

fn kill_group(pgid: i32, sig: i32) {
    unsafe {
        kill(-pgid, sig);
    }
}

pub struct ProcEntry {
    child: Child,
    pgid: i32,
    #[allow(dead_code)]
    log_path: PathBuf,
    #[allow(dead_code)]
    started_at: u64,
}

#[derive(Default)]
pub struct ProcMgr {
    procs: Mutex<HashMap<String, ProcEntry>>,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

impl ProcMgr {
    /// 以独立进程组启动任务,stdout/stderr 追加写入日志文件。
    /// 已在运行则返回 Err,保证同一任务不重复启动。
    pub fn spawn(&self, id: &str, mut cmd: Command, log_path: &Path) -> Result<(), String> {
        let mut map = self.procs.lock().unwrap();
        if let Some(e) = map.get_mut(id) {
            if matches!(e.child.try_wait(), Ok(None)) {
                return Err(format!("「{id}」已在运行中"));
            }
        }
        if let Some(parent) = log_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建日志目录失败: {e}"))?;
        }
        let mut log = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map_err(|e| format!("打开日志文件失败: {e}"))?;
        let _ = writeln!(log, "\n==== 启动于 {} (unix {}) ====", chrono_str(), now());
        cmd.stdin(Stdio::null())
            .stdout(log.try_clone().map_err(|e| e.to_string())?)
            .stderr(log)
            .process_group(0);
        let child = cmd.spawn().map_err(|e| format!("启动进程失败: {e}"))?;
        let pgid = child.id() as i32;
        map.insert(
            id.to_string(),
            ProcEntry {
                child,
                pgid,
                log_path: log_path.to_path_buf(),
                started_at: now(),
            },
        );
        Ok(())
    }

    pub fn is_running(&self, id: &str) -> bool {
        let mut map = self.procs.lock().unwrap();
        match map.get_mut(id) {
            Some(e) => matches!(e.child.try_wait(), Ok(None)),
            None => false,
        }
    }

    /// (running, pid)
    pub fn snapshot(&self, id: &str) -> Option<(bool, Option<u32>)> {
        let mut map = self.procs.lock().unwrap();
        let e = map.get_mut(id)?;
        let running = matches!(e.child.try_wait(), Ok(None));
        let pid = e.child.id();
        Some((running, if running { Some(pid) } else { None }))
    }

    pub fn stop(&self, id: &str) -> Result<(), String> {
        let mut e = {
            let mut map = self.procs.lock().unwrap();
            map.remove(id).ok_or_else(|| format!("「{id}」未在运行"))?
        };
        kill_group(e.pgid, SIGTERM);
        let deadline = Instant::now() + Duration::from_secs(4);
        loop {
            if matches!(e.child.try_wait(), Ok(Some(_))) {
                break;
            }
            if Instant::now() >= deadline {
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        kill_group(e.pgid, SIGKILL);
        let _ = e.child.wait();
        Ok(())
    }

    pub fn stop_all(&self) {
        let ids: Vec<String> = {
            let map = self.procs.lock().unwrap();
            map.keys().cloned().collect()
        };
        for id in ids {
            let _ = self.stop(&id);
        }
    }
}

fn chrono_str() -> String {
    let secs = now();
    let d = time_fmt(secs);
    d
}

/// 极简 UTC 时间格式化,避免引入 chrono 依赖。
fn time_fmt(secs: u64) -> String {
    let days = secs / 86400;
    let rem = secs % 86400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    // 1970-01-01 起的粗略日期(仅用于日志展示,不用于逻辑)
    let mut year = 1970i64;
    let mut dleft = days as i64;
    loop {
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let ydays = if leap { 366 } else { 365 };
        if dleft < ydays {
            break;
        }
        dleft -= ydays;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let mdays = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1;
    for &md in &mdays {
        if dleft < md {
            break;
        }
        dleft -= md;
        month += 1;
    }
    format!("{year}-{month:02}-{:02} {h:02}:{m:02}:{s:02} UTC", dleft + 1)
}

pub fn log_tail(path: &Path, max_bytes: usize) -> String {
    let data = match std::fs::read(path) {
        Ok(d) => d,
        Err(_) => return String::new(),
    };
    let start = data.len().saturating_sub(max_bytes);
    String::from_utf8_lossy(&data[start..]).into_owned()
}
