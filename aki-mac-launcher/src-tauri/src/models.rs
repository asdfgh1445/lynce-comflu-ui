use crate::config::{self, AppConfig};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelItem {
    pub name: String,
    pub path: String,
    pub size_mb: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelGroup {
    pub family: String,
    pub label: String,
    pub dir: String,
    pub count: usize,
    pub total_mb: f64,
    pub items: Vec<ModelItem>,
}

const MODEL_EXTS: [&str; 5] = ["safetensors", "ckpt", "pt", "gguf", "sft"];

fn family_label(family: &str) -> String {
    let label = match family {
        "Stable-Diffusion" => "大模型 Checkpoint",
        "Lora" | "lora" | "loras" => "LoRA",
        "VAE" | "vae" => "VAE",
        "text_encoders" => "文本编码器 CLIP/T5",
        "diffusion_models" => "扩散模型 (DiT/Flux)",
        "unet" => "UNet",
        "clip" => "CLIP",
        "clip_vision" => "CLIP Vision",
        "upscale_models" => "放大模型",
        "controlnet" => "ControlNet",
        "Embeddings" | "embeddings" => "Embeddings",
        "model_patches" => "Model Patches",
        "style_models" => "Style Models",
        other => other,
    };
    label.to_string()
}

fn walk(dir: &Path, depth: u32, items: &mut Vec<ModelItem>) {
    if depth > 4 {
        return;
    }
    let Ok(rd) = fs::read_dir(dir) else { return };
    for entry in rd.flatten() {
        let p = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        if p.is_dir() {
            walk(&p, depth + 1, items);
        } else if let Some(ext) = p.extension().and_then(|e| e.to_str()) {
            if MODEL_EXTS.contains(&ext.to_ascii_lowercase().as_str()) {
                let size = fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
                items.push(ModelItem {
                    name,
                    path: p.to_string_lossy().into_owned(),
                    size_mb: size as f64 / 1024.0 / 1024.0,
                });
            }
        }
    }
}

fn build_group(family: String, dir: PathBuf, mut items: Vec<ModelItem>) -> ModelGroup {
    items.sort_by(|a, b| b.size_mb.total_cmp(&a.size_mb));
    let count = items.len();
    let total: f64 = items.iter().map(|i| i.size_mb).sum();
    let label = family_label(&family);
    items.truncate(200);
    ModelGroup {
        family,
        label,
        dir: dir.to_string_lossy().into_owned(),
        count,
        total_mb: total,
        items,
    }
}

pub fn scan(cfg: &AppConfig) -> Vec<ModelGroup> {
    let mut groups = Vec::new();

    let models_root = PathBuf::from(config::expand_tilde(&cfg.swarm_dir)).join("Models");
    if let Ok(rd) = fs::read_dir(&models_root) {
        let mut subdirs: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        subdirs.sort();
        for sub in subdirs {
            let family = sub
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let mut items = Vec::new();
            walk(&sub, 0, &mut items);
            if !items.is_empty() {
                groups.push(build_group(family, sub, items));
            }
        }
    }

    for extra in &cfg.extra_model_dirs {
        let expanded = config::expand_tilde(extra);
        if expanded.is_empty() {
            continue;
        }
        let p = PathBuf::from(&expanded);
        let family = p
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| expanded.clone());
        let mut items = Vec::new();
        walk(&p, 0, &mut items);
        if !items.is_empty() {
            groups.push(build_group(family, p, items));
        }
    }

    groups.sort_by(|a, b| b.total_mb.total_cmp(&a.total_mb));
    groups
}
