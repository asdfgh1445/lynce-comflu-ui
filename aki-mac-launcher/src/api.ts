import { invoke } from "@tauri-apps/api/core";

export interface AppConfig {
  swarmDir: string;
  swarmPort: number;
  comfyDir: string;
  comfyPort: number;
  pythonBin: string;
  extraModelDirs: string[];
  extraEnv: string;
  swarmArgs: string;
  comfyArgs: string;
  autoOpen: boolean;
}

export interface AppInfo {
  id: string;
  name: string;
  running: boolean;
  pid: number | null;
  port: number;
  url: string;
  dir: string;
  dirValid: boolean;
  hint: string;
  portListening: boolean;
}

export interface ModelItem {
  name: string;
  path: string;
  sizeMb: number;
}

export interface ModelGroup {
  family: string;
  label: string;
  dir: string;
  count: number;
  totalMb: number;
  items: ModelItem[];
}

export interface ExtensionInfo {
  name: string;
  path: string;
  isGit: boolean;
}

export interface TaskInfo {
  running: boolean;
  pid: number | null;
  text: string;
}

export interface SysInfo {
  pythonVersion: string;
  comfyInstalled: boolean;
  swarmValid: boolean;
  freeGb: number;
}

export const api = {
  getConfig: () => invoke<AppConfig>("get_config"),
  saveConfig: (cfg: AppConfig) => invoke<void>("save_config", { cfg }),
  getApps: () => invoke<AppInfo[]>("get_apps"),
  startApp: (id: string) => invoke<void>("start_app", { id }),
  stopApp: (id: string) => invoke<void>("stop_app", { id }),
  getLogs: (id: string, bytes = 12000) =>
    invoke<TaskInfo>("get_logs", { id, bytes }),
  listModels: () => invoke<ModelGroup[]>("list_models"),
  installComfy: () => invoke<void>("install_comfy"),
  getTask: (id: string) => invoke<TaskInfo>("get_task", { id }),
  openUrl: (url: string) => invoke<void>("open_url", { url }),
  revealPath: (path: string) => invoke<void>("reveal_path", { path }),
  sysInfo: () => invoke<SysInfo>("sys_info"),
  upgradeApp: (id: string) => invoke<void>("upgrade_app", { id }),
  listExtensions: () => invoke<ExtensionInfo[]>("list_extensions"),
  installExtension: (url: string) => invoke<void>("install_extension", { url }),
};
