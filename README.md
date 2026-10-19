# dota2-mapping-config

Desktop app (Tauri + Svelte) to back up, share and apply Dota 2 hotkeys and settings.

- **Presets**: built-in LoL-like, Dota default, Legacy. Import/export preset `.json` files.
- **Key editor**: click a key, press a new one. Conflicts are highlighted.
- **Backups**: exact copies of the config files. One is made automatically before every apply or restore.

Every change is previewed as a diff before anything is written. Close Dota 2 before applying.

## Files touched

Under `Steam/userdata/<account>/570/`:

| File | Contents |
|---|---|
| `remote/cfg/dotakeys_personal.lst` | Hotkeys (Steam Cloud synced) |
| `local/cfg/user_convars_0_slot0.vcfg` | Game settings |
| `local/cfg/user_keys_0_slot0.vcfg` | Console binds |

## Preset format

Only listed entries change; everything else is left alone. `null` unbinds.

```json
{
  "name": "My preset",
  "description": "",
  "settings": { "UseHeroBindings": "0" },
  "keys": { "AbilityPrimary1": "Q", "AbilityPrimary1AutoCast": "ALT+Q", "ControlGroupCycle": null },
  "convars": { "dota_ability_quick_cast": "true" },
  "binds": {}
}
```

## Develop

```sh
pnpm install
pnpm tauri dev                     # run
cd src-tauri && cargo test         # core tests
pnpm tauri build                   # installer for the current OS
```
