use anyhow::{bail, Result};

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Str(String),
    Obj(Vec<(String, Value)>),
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            Value::Obj(_) => None,
        }
    }

    pub fn entries(&self) -> &[(String, Value)] {
        match self {
            Value::Obj(e) => e,
            Value::Str(_) => &[],
        }
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.entries().iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        match self {
            Value::Obj(e) => e.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v),
            Value::Str(_) => None,
        }
    }

    pub fn path(&self, path: &[&str]) -> Option<&Value> {
        path.iter().try_fold(self, |v, k| v.get(k))
    }

    pub fn path_mut(&mut self, path: &[&str]) -> Option<&mut Value> {
        path.iter().try_fold(self, |v, k| v.get_mut(k))
    }

    pub fn str_at(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }

    pub fn set_str(&mut self, key: &str, val: &str) {
        if let Some(v) = self.get_mut(key) {
            *v = Value::Str(val.to_string());
        } else if let Value::Obj(e) = self {
            e.push((key.to_string(), Value::Str(val.to_string())));
        }
    }

    pub fn remove(&mut self, key: &str) {
        if let Value::Obj(e) = self {
            e.retain(|(k, _)| !k.eq_ignore_ascii_case(key));
        }
    }
}

#[derive(Clone, Debug)]
pub struct Doc {
    pub root: Value,
    crlf: bool,
}

impl Doc {
    pub fn parse(text: &str) -> Result<Doc> {
        let mut p = Parser { s: text.as_bytes(), i: 0 };
        let entries = p.pairs(false)?;
        Ok(Doc { root: Value::Obj(entries), crlf: text.contains("\r\n") })
    }

    pub fn serialize(&self) -> String {
        let mut out = String::new();
        write_entries(&mut out, self.root.entries(), 0);
        if self.crlf { out.replace('\n', "\r\n") } else { out }
    }
}

fn write_entries(out: &mut String, entries: &[(String, Value)], depth: usize) {
    let indent = "\t".repeat(depth);
    for (k, v) in entries {
        match v {
            Value::Str(s) => out.push_str(&format!("{indent}\"{k}\"\t\t\"{s}\"\n")),
            Value::Obj(children) => {
                out.push_str(&format!("{indent}\"{k}\"\n{indent}{{\n"));
                write_entries(out, children, depth + 1);
                out.push_str(&format!("{indent}}}\n"));
            }
        }
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

enum Tok {
    Str(String),
    Open,
    Close,
    Eof,
}

impl Parser<'_> {
    fn pairs(&mut self, nested: bool) -> Result<Vec<(String, Value)>> {
        let mut out = Vec::new();
        loop {
            let key = match self.next()? {
                Tok::Str(k) => k,
                Tok::Close if nested => return Ok(out),
                Tok::Eof if !nested => return Ok(out),
                _ => bail!("unexpected token at byte {}", self.i),
            };
            let val = match self.next()? {
                Tok::Str(v) => Value::Str(v),
                Tok::Open => Value::Obj(self.pairs(true)?),
                _ => bail!("missing value for \"{key}\" at byte {}", self.i),
            };
            self.skip_conditional();
            out.push((key, val));
        }
    }

    fn skip_ws(&mut self) {
        loop {
            while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
                self.i += 1;
            }
            if self.s[self.i..].starts_with(b"//") {
                while self.i < self.s.len() && self.s[self.i] != b'\n' {
                    self.i += 1;
                }
            } else {
                return;
            }
        }
    }

    fn skip_conditional(&mut self) {
        self.skip_ws();
        if self.s.get(self.i) == Some(&b'[') {
            while self.i < self.s.len() && self.s[self.i] != b']' {
                self.i += 1;
            }
            self.i += 1;
        }
    }

    // Valve writes these files without escaping, e.g. a backslash key is stored as "\".
    fn next(&mut self) -> Result<Tok> {
        self.skip_ws();
        let Some(&c) = self.s.get(self.i) else { return Ok(Tok::Eof) };
        self.i += 1;
        match c {
            b'{' => Ok(Tok::Open),
            b'}' => Ok(Tok::Close),
            b'"' => {
                let start = self.i;
                while self.i < self.s.len() && self.s[self.i] != b'"' {
                    self.i += 1;
                }
                if self.i >= self.s.len() {
                    bail!("unterminated string at byte {start}");
                }
                self.i += 1;
                Ok(Tok::Str(String::from_utf8_lossy(&self.s[start..self.i - 1]).into_owned()))
            }
            _ => {
                let start = self.i - 1;
                while self.i < self.s.len() && !self.s[self.i].is_ascii_whitespace() && !b"{}\"".contains(&self.s[self.i]) {
                    self.i += 1;
                }
                Ok(Tok::Str(String::from_utf8_lossy(&self.s[start..self.i]).into_owned()))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_valve_format() {
        let text = "\"config\"\n{\n\t\"bindings\"\n\t{\n\t\t\"\\\"\t\t\"<unbound>\"\n\t\t\"F10\"\t\t\"<unbound>\"\n\t}\n}\n";
        let doc = Doc::parse(text).unwrap();
        assert_eq!(doc.serialize(), text);
        assert_eq!(doc.root.path(&["config", "bindings", "\\"]).and_then(Value::as_str), Some("<unbound>"));
    }

    #[test]
    fn handles_comments_unquoted_and_crlf() {
        let doc = Doc::parse("// hi\r\nroot\r\n{\r\n\tkey val [$WIN32]\r\n}\r\n").unwrap();
        assert_eq!(doc.root.path(&["root", "key"]).and_then(Value::as_str), Some("val"));
        assert_eq!(doc.serialize(), "\"root\"\r\n{\r\n\t\"key\"\t\t\"val\"\r\n}\r\n");
    }

    #[test]
    fn rejects_unbalanced() {
        assert!(Doc::parse("\"a\"\n{\n\t\"b\"\t\"c\"\n").is_err());
    }
}
