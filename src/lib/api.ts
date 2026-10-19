import { invoke } from "@tauri-apps/api/core";

export type Account = { id: number; name: string | null; dir: string };
export type Preset = {
  name: string;
  description: string;
  settings: Record<string, string>;
  keys: Record<string, string | null>;
  convars: Record<string, string>;
  binds: Record<string, string | null>;
};
export type Change = { section: "setting" | "key" | "convar" | "bind"; id: string; from: string | null; to: string | null };
export type Plan = { changes: Change[]; warnings: string[] };
export type Binding = { id: string; binding: string | null; panel: string | null };
export type Snapshot = { id: string; label: string; created_ms: number };

export const emptyPreset = (name: string): Preset => ({ name, description: "", settings: {}, keys: {}, convars: {}, binds: {} });

export const api = {
  accounts: () => invoke<Account[]>("accounts"),
  dotaRunning: () => invoke<boolean>("dota_running"),
  bundledPresets: () => invoke<Preset[]>("bundled_presets"),
  readPreset: (path: string) => invoke<Preset>("read_preset", { path }),
  writePreset: (path: string, preset: Preset) => invoke<void>("write_preset", { path, preset }),
  bindings: (account: number) => invoke<Binding[]>("bindings", { account }),
  exportCurrent: (account: number, name: string) => invoke<Preset>("export_current", { account, name }),
  preview: (account: number, preset: Preset) => invoke<Plan>("preview", { account, preset }),
  apply: (account: number, preset: Preset) => invoke<Plan>("apply", { account, preset }),
  snapshots: (account: number) => invoke<Snapshot[]>("snapshots", { account }),
  createSnapshot: (account: number, label: string) => invoke<Snapshot>("create_snapshot", { account, label }),
  restoreSnapshot: (account: number, id: string) => invoke<void>("restore_snapshot", { account, id }),
  deleteSnapshot: (account: number, id: string) => invoke<void>("delete_snapshot", { account, id }),
};
