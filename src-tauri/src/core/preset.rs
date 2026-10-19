use super::files::Configs;
use super::kv::Value;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

const SETTINGS: [&str; 5] = ["UseHeroBindings", "LinkHeroToCourier", "LinkHeroToNeutral", "LinkPlayerToSpectator", "PerUnitKeybinds"];
const MODIFIERS: [&str; 3] = ["SHIFT", "CTRL", "ALT"];
const UNBOUND: &str = "<unbound>";
const OTHER_CONTEXTS: [&str; 6] = ["Spectator", "ShopTab", "ShopSlot", "Legacy", "TalentUpgrade", "NeutralItemSelect"];

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Preset {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub settings: BTreeMap<String, String>,
    #[serde(default)]
    pub keys: BTreeMap<String, Option<String>>,
    #[serde(default)]
    pub convars: BTreeMap<String, String>,
    #[serde(default)]
    pub binds: BTreeMap<String, Option<String>>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Change {
    pub section: &'static str,
    pub id: String,
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Plan {
    pub changes: Vec<Change>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct Binding {
    pub id: String,
    pub binding: Option<String>,
    pub panel: Option<String>,
}

pub fn normalize(binding: &str) -> Option<String> {
    let b = binding.trim();
    if b.is_empty() {
        return None;
    }
    match b.split_once('+') {
        Some((m, k)) if !k.is_empty() => {
            let m = m.trim().to_ascii_uppercase();
            let m = if m == "CONTROL" { "CTRL".to_string() } else { m };
            Some(if MODIFIERS.contains(&m.as_str()) { format!("{m}+{}", k.trim().to_ascii_uppercase()) } else { b.to_ascii_uppercase() })
        }
        _ => Some(b.to_ascii_uppercase()),
    }
}

fn binding_of(block: &Value) -> Option<String> {
    let key = block.str_at("Key").filter(|k| !k.is_empty())?;
    match block.str_at("Modifier").filter(|m| !m.is_empty() && !m.eq_ignore_ascii_case("None")) {
        Some(m) => normalize(&format!("{m}+{key}")),
        None => normalize(key),
    }
}

fn set_binding(block: &mut Value, binding: Option<&str>) {
    match binding.map(|b| b.split_once('+').filter(|(m, _)| MODIFIERS.contains(m)).unwrap_or(("", b))) {
        None => {
            block.remove("Key");
            block.remove("Modifier");
        }
        Some((m, k)) => {
            block.set_str("Key", k);
            if m.is_empty() { block.remove("Modifier") } else { block.set_str("Modifier", m) }
        }
    }
}

pub fn apply(cfg: &mut Configs, p: &Preset) -> Plan {
    let mut plan = Plan::default();
    match cfg.keys.root.get_mut("KeyBindings") {
        Some(kb) => apply_keybindings(kb, p, &mut plan),
        None => plan.warnings.push("Keybinding file has no KeyBindings section; hotkeys skipped".into()),
    }
    if let Some(convars) = cfg.convars.root.path_mut(&["config", "convars"]) {
        for (name, want) in &p.convars {
            let cur = convars.str_at(name).map(String::from);
            if cur.as_deref() != Some(want) {
                convars.set_str(name, want);
                plan.changes.push(Change { section: "convar", id: name.clone(), from: cur, to: Some(want.clone()) });
            }
        }
    }
    if let Some(binds) = cfg.binds.root.path_mut(&["config", "bindings"]) {
        for (key, want) in &p.binds {
            let cur = binds.str_at(key).filter(|v| *v != UNBOUND).map(String::from);
            if cur != *want {
                binds.set_str(key, want.as_deref().unwrap_or(UNBOUND));
                plan.changes.push(Change { section: "bind", id: key.clone(), from: cur, to: want.clone() });
            }
        }
    }
    plan
}

fn apply_keybindings(kb: &mut Value, p: &Preset, plan: &mut Plan) {
    for (name, want) in &p.settings {
        match kb.str_at(name).map(String::from) {
            None => plan.warnings.push(format!("Unknown setting {name}, skipped")),
            Some(cur) if cur != *want => {
                kb.set_str(name, want);
                plan.changes.push(Change { section: "setting", id: name.clone(), from: Some(cur), to: Some(want.clone()) });
            }
            Some(_) => {}
        }
    }
    let Some(keys) = kb.get_mut("Keys") else { return };
    let mut changed = BTreeSet::new();
    for (action, want) in &p.keys {
        let Some(block @ Value::Obj(_)) = keys.get_mut(action) else {
            plan.warnings.push(format!("Unknown action {action}, skipped"));
            continue;
        };
        let want = want.as_deref().and_then(normalize);
        let cur = binding_of(block);
        if cur != want {
            set_binding(block, want.as_deref());
            changed.insert(action.clone());
            plan.changes.push(Change { section: "key", id: action.clone(), from: cur, to: want });
        }
    }
    plan.warnings.extend(conflicts(keys, &changed));
}

fn conflicts(keys: &Value, changed: &BTreeSet<String>) -> Vec<String> {
    let mut by_key: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    for (id, block) in keys.entries() {
        if OTHER_CONTEXTS.iter().any(|c| id.starts_with(c)) {
            continue;
        }
        if let Some(b) = binding_of(block) {
            by_key.entry(b).or_default().push(id);
        }
    }
    by_key
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1 && ids.iter().any(|i| changed.contains(*i)))
        .map(|(k, ids)| format!("{k} is bound to {}", ids.join(", ")))
        .collect()
}

pub fn from_current(cfg: &Configs, name: &str) -> Preset {
    let kb = cfg.keys.root.get("KeyBindings");
    let settings = SETTINGS.iter().filter_map(|s| Some((s.to_string(), kb?.str_at(s)?.to_string()))).collect();
    let convars = entries(&cfg.convars.root, &["config", "convars"]).filter_map(|(k, v)| Some((k.clone(), v.as_str()?.to_string()))).collect();
    let binds = entries(&cfg.binds.root, &["config", "bindings"])
        .filter_map(|(k, v)| Some((k.clone(), v.as_str().map(|v| Some(v.to_string()).filter(|v| v != UNBOUND))?)))
        .collect();
    Preset {
        name: name.into(),
        description: "Exported from current Dota 2 settings".into(),
        settings,
        keys: bindings(cfg).into_iter().map(|b| (b.id, b.binding)).collect(),
        convars,
        binds,
    }
}

pub fn bindings(cfg: &Configs) -> Vec<Binding> {
    entries(&cfg.keys.root, &["KeyBindings", "Keys"])
        .filter(|(id, v)| matches!(v, Value::Obj(_)) && id != "Heroes")
        .map(|(id, v)| Binding { id: id.clone(), binding: binding_of(v), panel: v.str_at("Panel").map(String::from) })
        .collect()
}

fn entries<'a>(root: &'a Value, path: &[&str]) -> impl Iterator<Item = &'a (String, Value)> {
    root.path(path).map(Value::entries).unwrap_or_default().iter()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static N: AtomicUsize = AtomicUsize::new(0);

    const KEYS: &str = "\"KeyBindings\"\n{\n\t\"UseHeroBindings\"\t\t\"0\"\n\t\"Keys\"\n\t{\n\t\t\"AbilityPrimary1\"\n\t\t{\n\t\t\t\"Name\"\t\t\"AbilityPrimary1\"\n\t\t\t\"Key\"\t\t\"Q\"\n\t\t}\n\t\t\"AbilityPrimary1AutoCast\"\n\t\t{\n\t\t\t\"Key\"\t\t\"q\"\n\t\t\t\"Modifier\"\t\t\"ALT\"\n\t\t}\n\t\t\"Inventory1\"\n\t\t{\n\t\t\t\"Key\"\t\t\"Z\"\n\t\t}\n\t\t\"HeroStop\"\n\t\t{\n\t\t\t\"Key\"\t\t\"S\"\n\t\t}\n\t\t\"SpectatorStop\"\n\t\t{\n\t\t\t\"Key\"\t\t\"1\"\n\t\t}\n\t\t\"Heroes\"\n\t\t{\n\t\t\t\"Version\"\t\t\"35\"\n\t\t}\n\t}\n}\n";

    fn cfg() -> Configs {
        let dir = std::env::temp_dir().join(format!("dotacfg-test-{}-{}", std::process::id(), N.fetch_add(1, Ordering::SeqCst)));
        crate::core::files::write_atomic(&dir.join(crate::core::files::KEYS), KEYS).unwrap();
        Configs::load(&dir).unwrap()
    }

    fn preset(keys: &[(&str, Option<&str>)]) -> Preset {
        Preset { keys: keys.iter().map(|(k, v)| (k.to_string(), v.map(String::from))).collect(), ..Default::default() }
    }

    #[test]
    fn normalizes_bindings() {
        assert_eq!(normalize("alt+q").as_deref(), Some("ALT+Q"));
        assert_eq!(normalize("control+1").as_deref(), Some("CTRL+1"));
        assert_eq!(normalize("SHIFT++").as_deref(), Some("SHIFT++"));
        assert_eq!(normalize(" "), None);
    }

    #[test]
    fn applies_only_real_changes() {
        let mut c = cfg();
        let plan = apply(&mut c, &preset(&[("AbilityPrimary1AutoCast", Some("ALT+Q")), ("Inventory1", Some("1")), ("HeroStop", None), ("Nope", Some("X"))]));
        assert_eq!(plan.changes.iter().map(|c| c.id.as_str()).collect::<Vec<_>>(), ["HeroStop", "Inventory1"]);
        assert_eq!(plan.warnings, ["Unknown action Nope, skipped"]);
        let keys = c.keys.root.path(&["KeyBindings", "Keys"]).unwrap();
        assert_eq!(keys.path(&["Inventory1", "Key"]).and_then(Value::as_str), Some("1"));
        assert!(keys.path(&["HeroStop", "Key"]).is_none());
    }

    #[test]
    fn sets_and_clears_modifier() {
        let mut c = cfg();
        apply(&mut c, &preset(&[("AbilityPrimary1AutoCast", Some("W")), ("Inventory1", Some("shift+z"))]));
        let keys = c.keys.root.path(&["KeyBindings", "Keys"]).unwrap();
        assert!(keys.path(&["AbilityPrimary1AutoCast", "Modifier"]).is_none());
        assert_eq!(keys.path(&["Inventory1", "Modifier"]).and_then(Value::as_str), Some("SHIFT"));
    }

    #[test]
    fn warns_on_conflicts_ignoring_other_contexts() {
        let mut c = cfg();
        let plan = apply(&mut c, &preset(&[("Inventory1", Some("Q")), ("HeroStop", Some("1"))]));
        assert_eq!(plan.warnings, ["Q is bound to AbilityPrimary1, Inventory1"]);
    }

    #[test]
    fn export_round_trips() {
        let mut c = cfg();
        let p = from_current(&c, "mine");
        assert_eq!(p.settings.get("UseHeroBindings").map(String::as_str), Some("0"));
        assert!(!p.keys.contains_key("Heroes"));
        assert!(apply(&mut c, &p).changes.is_empty());
    }
}
