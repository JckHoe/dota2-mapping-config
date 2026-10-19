use crate::core::{self, preset::{Binding, Plan, Preset}, snapshot::Snapshot, steam::Account, Service};
use std::path::PathBuf;
use tauri::State;

type Res<T> = Result<T, String>;

fn err(e: anyhow::Error) -> String {
    format!("{e:#}")
}

#[tauri::command]
pub fn accounts(svc: State<Service>) -> Res<Vec<Account>> {
    svc.accounts().map_err(err)
}

#[tauri::command]
pub fn dota_running() -> bool {
    core::dota_running()
}

#[tauri::command]
pub fn bundled_presets() -> Vec<Preset> {
    core::bundled_presets()
}

#[tauri::command]
pub fn read_preset(path: PathBuf) -> Res<Preset> {
    core::read_preset(&path).map_err(err)
}

#[tauri::command]
pub fn write_preset(path: PathBuf, preset: Preset) -> Res<()> {
    core::write_preset(&path, &preset).map_err(err)
}

#[tauri::command]
pub fn bindings(svc: State<Service>, account: u32) -> Res<Vec<Binding>> {
    svc.bindings(account).map_err(err)
}

#[tauri::command]
pub fn export_current(svc: State<Service>, account: u32, name: String) -> Res<Preset> {
    svc.export(account, &name).map_err(err)
}

#[tauri::command]
pub fn preview(svc: State<Service>, account: u32, preset: Preset) -> Res<Plan> {
    svc.preview(account, &preset).map_err(err)
}

#[tauri::command]
pub fn apply(svc: State<Service>, account: u32, preset: Preset) -> Res<Plan> {
    svc.apply(account, &preset).map_err(err)
}

#[tauri::command]
pub fn snapshots(svc: State<Service>, account: u32) -> Res<Vec<Snapshot>> {
    svc.snapshots(account).map_err(err)
}

#[tauri::command]
pub fn create_snapshot(svc: State<Service>, account: u32, label: String) -> Res<Snapshot> {
    svc.create_snapshot(account, &label).map_err(err)
}

#[tauri::command]
pub fn restore_snapshot(svc: State<Service>, account: u32, id: String) -> Res<()> {
    svc.restore_snapshot(account, &id).map_err(err)
}

#[tauri::command]
pub fn delete_snapshot(svc: State<Service>, account: u32, id: String) -> Res<()> {
    svc.delete_snapshot(account, &id).map_err(err)
}
