use super::kv::Doc;
use anyhow::{Context, Result};
use std::path::Path;

pub const KEYS: &str = "remote/cfg/dotakeys_personal.lst";
pub const CONVARS: &str = "local/cfg/user_convars_0_slot0.vcfg";
pub const BINDS: &str = "local/cfg/user_keys_0_slot0.vcfg";
pub const ALL: [&str; 3] = [KEYS, CONVARS, BINDS];

const EMPTY_CONVARS: &str = "\"config\"\n{\n\t\"convars\"\n\t{\n\t}\n}\n";
const EMPTY_BINDS: &str = "\"config\"\n{\n\t\"bindings\"\n\t{\n\t}\n}\n";

pub struct Configs {
    pub keys: Doc,
    pub convars: Doc,
    pub binds: Doc,
    original: [String; 3],
}

impl Configs {
    pub fn load(dir: &Path) -> Result<Configs> {
        let keys = std::fs::read_to_string(dir.join(KEYS)).with_context(|| format!("reading {KEYS}. Launch Dota once so it creates it"))?;
        let convars = std::fs::read_to_string(dir.join(CONVARS)).unwrap_or_else(|_| EMPTY_CONVARS.into());
        let binds = std::fs::read_to_string(dir.join(BINDS)).unwrap_or_else(|_| EMPTY_BINDS.into());
        Ok(Configs {
            keys: Doc::parse(&keys).with_context(|| format!("parsing {KEYS}"))?,
            convars: Doc::parse(&convars).with_context(|| format!("parsing {CONVARS}"))?,
            binds: Doc::parse(&binds).with_context(|| format!("parsing {BINDS}"))?,
            original: [keys, convars, binds],
        })
    }

    pub fn save(&self, dir: &Path) -> Result<()> {
        let docs = [&self.keys, &self.convars, &self.binds];
        for ((rel, doc), orig) in ALL.iter().zip(docs).zip(&self.original) {
            let text = doc.serialize();
            if text != *orig {
                write_atomic(&dir.join(rel), &text)?;
            }
        }
        Ok(())
    }
}

pub fn write_atomic(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("dotacfg-tmp");
    std::fs::write(&tmp, text)?;
    std::fs::rename(&tmp, path).with_context(|| format!("writing {}", path.display()))
}
