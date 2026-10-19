use super::files;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub label: String,
    pub created_ms: u128,
}

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf, account_id: u32) -> Store {
        Store { root: root.join(account_id.to_string()) }
    }

    pub fn create(&self, dota_dir: &Path, label: &str) -> Result<Snapshot> {
        let mut created_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
        while self.root.join(created_ms.to_string()).exists() {
            created_ms += 1;
        }
        let snap = Snapshot { id: created_ms.to_string(), label: label.to_string(), created_ms };
        let dir = self.root.join(&snap.id);
        for rel in files::ALL {
            let src = dota_dir.join(rel);
            if src.exists() {
                let dst = dir.join(rel);
                std::fs::create_dir_all(dst.parent().unwrap())?;
                std::fs::copy(&src, &dst).with_context(|| format!("backing up {rel}"))?;
            }
        }
        std::fs::create_dir_all(&dir)?;
        std::fs::write(dir.join("meta.json"), serde_json::to_string_pretty(&snap)?)?;
        Ok(snap)
    }

    pub fn list(&self) -> Result<Vec<Snapshot>> {
        let Ok(entries) = std::fs::read_dir(&self.root) else { return Ok(vec![]) };
        let mut out: Vec<Snapshot> = entries
            .flatten()
            .filter_map(|e| serde_json::from_slice(&std::fs::read(e.path().join("meta.json")).ok()?).ok())
            .collect();
        out.sort_by(|a, b| b.created_ms.cmp(&a.created_ms));
        Ok(out)
    }

    pub fn restore(&self, dota_dir: &Path, id: &str) -> Result<()> {
        let dir = self.dir(id)?;
        for rel in files::ALL {
            let src = dir.join(rel);
            if src.exists() {
                files::write_atomic(&dota_dir.join(rel), &std::fs::read_to_string(src)?)?;
            }
        }
        Ok(())
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        Ok(std::fs::remove_dir_all(self.dir(id)?)?)
    }

    fn dir(&self, id: &str) -> Result<PathBuf> {
        if id.is_empty() || !id.bytes().all(|b| b.is_ascii_digit()) {
            bail!("invalid snapshot id");
        }
        let dir = self.root.join(id);
        if !dir.join("meta.json").exists() {
            bail!("snapshot {id} not found");
        }
        Ok(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_then_restore() {
        let tmp = std::env::temp_dir().join(format!("dotacfg-snap-{}", std::process::id()));
        let dota = tmp.join("dota");
        files::write_atomic(&dota.join(files::KEYS), "a").unwrap();
        let store = Store::new(tmp.join("snaps"), 1);
        let snap = store.create(&dota, "before").unwrap();
        files::write_atomic(&dota.join(files::KEYS), "b").unwrap();
        store.restore(&dota, &snap.id).unwrap();
        assert_eq!(std::fs::read_to_string(dota.join(files::KEYS)).unwrap(), "a");
        assert_eq!(store.list().unwrap()[0].label, "before");
        assert!(store.restore(&dota, "../x").is_err());
        store.delete(&snap.id).unwrap();
        assert!(store.list().unwrap().is_empty());
    }
}
