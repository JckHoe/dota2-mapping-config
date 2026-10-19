use super::kv::{Doc, Value};
use anyhow::{anyhow, Result};
use serde::Serialize;
use std::path::{Path, PathBuf};

const DOTA_APP_ID: &str = "570";
const STEAM_ID64_BASE: u64 = 76561197960265728;

#[derive(Clone, Debug, Serialize)]
pub struct Account {
    pub id: u32,
    pub name: Option<String>,
    pub dir: PathBuf,
}

pub fn accounts() -> Result<Vec<Account>> {
    let roots = steam_roots();
    let mut out = Vec::new();
    for root in &roots {
        let names = persona_names(root);
        let Ok(entries) = std::fs::read_dir(root.join("userdata")) else { continue };
        for e in entries.flatten() {
            let Ok(id) = e.file_name().to_string_lossy().parse::<u32>() else { continue };
            let dir = e.path().join(DOTA_APP_ID);
            if dir.is_dir() && !out.iter().any(|a: &Account| a.id == id) {
                let name = names.iter().find(|(i, _)| *i == id).map(|(_, n)| n.clone());
                out.push(Account { id, name, dir });
            }
        }
    }
    if out.is_empty() {
        return Err(anyhow!("No Dota 2 config found. Looked in: {}", roots.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", ")));
    }
    Ok(out)
}

pub fn account(id: u32) -> Result<Account> {
    accounts()?.into_iter().find(|a| a.id == id).ok_or_else(|| anyhow!("Steam account {id} not found"))
}

fn persona_names(root: &Path) -> Vec<(u32, String)> {
    let Ok(text) = std::fs::read_to_string(root.join("config/loginusers.vdf")) else { return vec![] };
    let Ok(doc) = Doc::parse(&text) else { return vec![] };
    doc.root
        .get("users")
        .map(Value::entries)
        .unwrap_or_default()
        .iter()
        .filter_map(|(id64, v)| {
            let id = id64.parse::<u64>().ok()?.checked_sub(STEAM_ID64_BASE)?;
            Some((u32::try_from(id).ok()?, v.str_at("PersonaName")?.to_string()))
        })
        .collect()
}

fn steam_roots() -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = Vec::new();
    for p in candidates() {
        if let Ok(c) = p.canonicalize() {
            if c.join("userdata").is_dir() && !roots.contains(&c) {
                roots.push(c);
            }
        }
    }
    roots
}

#[cfg(target_os = "macos")]
fn candidates() -> Vec<PathBuf> {
    home().map(|h| vec![h.join("Library/Application Support/Steam")]).unwrap_or_default()
}

#[cfg(target_os = "linux")]
fn candidates() -> Vec<PathBuf> {
    let Some(h) = home() else { return vec![] };
    vec![
        h.join(".steam/steam"),
        h.join(".local/share/Steam"),
        h.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        h.join("snap/steam/common/.local/share/Steam"),
    ]
}

#[cfg(windows)]
fn candidates() -> Vec<PathBuf> {
    use winreg::{enums::HKEY_CURRENT_USER, RegKey};
    let mut out = Vec::new();
    if let Ok(path) = RegKey::predef(HKEY_CURRENT_USER).open_subkey("Software\\Valve\\Steam").and_then(|k| k.get_value::<String, _>("SteamPath")) {
        out.push(PathBuf::from(path));
    }
    for var in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Ok(pf) = std::env::var(var) {
            out.push(PathBuf::from(pf).join("Steam"));
        }
    }
    out
}

#[cfg(not(windows))]
fn home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}
