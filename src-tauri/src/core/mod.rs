pub mod files;
pub mod kv;
pub mod preset;
pub mod snapshot;
pub mod steam;

use anyhow::{bail, Context, Result};
use files::Configs;
use preset::{Binding, Plan, Preset};
use snapshot::{Snapshot, Store};
use std::path::{Path, PathBuf};

const BUNDLED: [&str; 3] = [
    include_str!("../../presets/lol.json"),
    include_str!("../../presets/dota-default.json"),
    include_str!("../../presets/legacy.json"),
];

pub fn bundled_presets() -> Vec<Preset> {
    BUNDLED.iter().map(|p| serde_json::from_str(p).expect("bundled preset is valid")).collect()
}

pub fn read_preset(path: &Path) -> Result<Preset> {
    let text = std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    serde_json::from_str(&text).context("not a valid preset file")
}

pub fn write_preset(path: &Path, p: &Preset) -> Result<()> {
    files::write_atomic(path, &serde_json::to_string_pretty(p)?)
}

pub fn dota_running() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.processes().values().any(|p| {
        let name = p.name().to_string_lossy().to_ascii_lowercase();
        name == "dota2" || name == "dota2.exe"
    })
}

pub struct Service {
    snapshots_root: PathBuf,
}

impl Service {
    pub fn new(snapshots_root: PathBuf) -> Service {
        Service { snapshots_root }
    }

    pub fn accounts(&self) -> Result<Vec<steam::Account>> {
        steam::accounts()
    }

    pub fn bindings(&self, account: u32) -> Result<Vec<Binding>> {
        Ok(preset::bindings(&Configs::load(&steam::account(account)?.dir)?))
    }

    pub fn export(&self, account: u32, name: &str) -> Result<Preset> {
        Ok(preset::from_current(&Configs::load(&steam::account(account)?.dir)?, name))
    }

    pub fn preview(&self, account: u32, p: &Preset) -> Result<Plan> {
        let mut cfg = Configs::load(&steam::account(account)?.dir)?;
        Ok(preset::apply(&mut cfg, p))
    }

    pub fn apply(&self, account: u32, p: &Preset) -> Result<Plan> {
        ensure_dota_closed()?;
        let dir = steam::account(account)?.dir;
        let mut cfg = Configs::load(&dir)?;
        let plan = preset::apply(&mut cfg, p);
        if !plan.changes.is_empty() {
            self.store(account).create(&dir, &format!("Before applying {}", p.name))?;
            cfg.save(&dir)?;
        }
        Ok(plan)
    }

    pub fn snapshots(&self, account: u32) -> Result<Vec<Snapshot>> {
        self.store(account).list()
    }

    pub fn create_snapshot(&self, account: u32, label: &str) -> Result<Snapshot> {
        self.store(account).create(&steam::account(account)?.dir, label)
    }

    pub fn restore_snapshot(&self, account: u32, id: &str) -> Result<()> {
        ensure_dota_closed()?;
        let dir = steam::account(account)?.dir;
        let store = self.store(account);
        store.create(&dir, "Before restore")?;
        store.restore(&dir, id)
    }

    pub fn delete_snapshot(&self, account: u32, id: &str) -> Result<()> {
        self.store(account).delete(id)
    }

    fn store(&self, account: u32) -> Store {
        Store::new(self.snapshots_root.clone(), account)
    }
}

fn ensure_dota_closed() -> Result<()> {
    if dota_running() {
        bail!("Close Dota 2 first, otherwise it overwrites these files when it exits");
    }
    Ok(())
}
