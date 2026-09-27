//! Structured data: JSON, YAML, TOML and XML, all through `serde_json::Value`
//! (with key order preserved).

use serde_json::{Map, Number, Value};

use crate::xml::{self, Element, Node};

pub fn convert(input: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    let value = read(input, from)?;
    write(&value, to)
}

fn utf8(input: &[u8]) -> Result<&str, String> {
    let s = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    Ok(s.strip_prefix('\u{feff}').unwrap_or(s))
}

pub fn read(input: &[u8], from: &str) -> Result<Value, String> {
    let text = utf8(input)?;
    match from {
        "json" => serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {e}")),
        "yaml" => read_yaml(text),
        "toml" => {
            let v: toml::Value = toml::from_str(text).map_err(|e| format!("Invalid TOML: {e}"))?;
            Ok(from_toml(v))
        }
        "xml" => Ok(from_xml(&xml::parse(text)?)),
        _ => Err(format!("Unsupported data format: {from}")),
    }
}

pub fn write(value: &Value, to: &str) -> Result<Vec<u8>, String> {
    let mut out = match to {
        "json" => serde_json::to_string_pretty(value).map_err(|e| e.to_string())?,
        "yaml" => serde_yaml_ng::to_string(value).map_err(|e| format!("YAML error: {e}"))?,
        "toml" => write_toml(value)?,
        "xml" => write_xml(value),
        _ => return Err(format!("Unsupported data output: {to}")),
    };
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out.into_bytes())
}

// ── YAML ────────────────────────────────────────────────

fn read_yaml(text: &str) -> Result<Value, String> {
    use serde::Deserialize;
    let mut docs = Vec::new();
    for doc in serde_yaml_ng::Deserializer::from_str(text) {
        let v = serde_yaml_ng::Value::deserialize(doc).map_err(|e| format!("Invalid YAML: {e}"))?;
        docs.push(from_yaml(v));
    }
    Ok(match docs.len() {
        0 => Value::Null,
        1 => docs.pop().expect("one"),
        // A multi-document stream becomes an array of documents.
        _ => Value::Array(docs),
    })
}

fn from_yaml(v: serde_yaml_ng::Value) -> Value {
    use serde_yaml_ng::Value as Y;
    match v {
        Y::Null => Value::Null,
        Y::Bool(b) => Value::Bool(b),
        Y::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::from(i)
            } else if let Some(u) = n.as_u64() {
                Value::from(u)
            } else {
                let f = n.as_f64().unwrap_or(f64::NAN);
                Number::from_f64(f)
                    .map(Value::Number)
                    .unwrap_or_else(|| Value::String(n.to_string()))
            }
        }
        Y::String(s) => Value::String(s),
        Y::Sequence(seq) => Value::Array(seq.into_iter().map(from_yaml).collect()),
        Y::Mapping(m) => {
            let mut out = Map::new();
            for (k, v) in m {
                let key = match from_yaml(k) {
                    Value::String(s) => s,
                    Value::Null => "null".to_string(),
                    other => other.to_string(),
                };
                out.insert(key, from_yaml(v));
            }
            Value::Object(out)
        }
        Y::Tagged(t) => from_yaml(t.value),
    }
}

// ── TOML ────────────────────────────────────────────────

fn from_toml(v: toml::Value) -> Value {
    match v {
        toml::Value::String(s) => Value::String(s),
        toml::Value::Integer(i) => Value::from(i),
        toml::Value::Float(f) => Number::from_f64(f)
            .map(Value::Number)
            .unwrap_or_else(|| Value::String(f.to_string())),
        toml::Value::Boolean(b) => Value::Bool(b),
        toml::Value::Datetime(d) => Value::String(d.to_string()),
        toml::Value::Array(a) => Value::Array(a.into_iter().map(from_toml).collect()),
        toml::Value::Table(t) => {
            Value::Object(t.into_iter().map(|(k, v)| (k, from_toml(v))).collect())
        }
    }
}

/// TOML has no null, so null values are left out.
fn strip_nulls(v: &Value) -> Option<Value> {
    match v {
        Value::Null => None,
        Value::Array(a) => Some(Value::Array(a.iter().filter_map(strip_nulls).collect())),
        Value::Object(o) => Some(Value::Object(
            o.iter()
                .filter_map(|(k, v)| strip_nulls(v).map(|v| (k.clone(), v)))
                .collect(),
        )),
        other => Some(other.clone()),
    }
}

fn write_toml(value: &Value) -> Result<String, String> {
    let cleaned = strip_nulls(value).unwrap_or(Value::Object(Map::new()));
    // TOML documents are tables; wrap anything else.
    let root = match cleaned {
        Value::Object(_) => cleaned,
        other => {
            let mut m = Map::new();
            m.insert("items".into(), other);
            Value::Object(m)
        }
    };
    toml::to_string_pretty(&root).map_err(|e| format!("Cannot express as TOML: {e}"))
}

// ── XML ─────────────────────────────────────────────────
//
// Mapping (the common "xml2js" convention): an element becomes an object,
// attributes are `@name` keys, mixed text is `#text`, repeated children become
// arrays, and an element with only text becomes a string.

fn from_xml(root: &Element) -> Value {
    let mut m = Map::new();
    m.insert(root.name.clone(), element_value(root));
    Value::Object(m)
}

fn element_value(el: &Element) -> Value {
    let text: String = el
        .children
        .iter()
        .filter_map(|n| match n {
            Node::Text(t) => Some(t.as_str()),
            Node::Elem(_) => None,
        })
        .collect();
    let text = text.trim();
    let has_children = el.elements().next().is_some();
    if el.attrs.is_empty() && !has_children {
        return Value::String(text.to_string());
    }
    let mut m = Map::new();
    for (k, v) in &el.attrs {
        if k.starts_with("xmlns") {
            continue;
        }
        m.insert(format!("@{k}"), Value::String(v.clone()));
    }
    for child in el.elements() {
        let v = element_value(child);
        match m.get_mut(&child.name) {
            Some(Value::Array(a)) => a.push(v),
            Some(existing) => {
                let prev = existing.take();
                *existing = Value::Array(vec![prev, v]);
            }
            None => {
                m.insert(child.name.clone(), v);
            }
        }
    }
    if !text.is_empty() {
        m.insert("#text".into(), Value::String(text.to_string()));
    }
    Value::Object(m)
}

fn xml_name(key: &str) -> String {
    let mut s: String = key
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | ':') {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() || !s.starts_with(|c: char| c.is_alphabetic() || c == '_') {
        s.insert(0, '_');
    }
    s
}

fn scalar_text(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn write_element(out: &mut String, name: &str, v: &Value, depth: usize) {
    let pad = "  ".repeat(depth);
    let name = xml_name(name);
    match v {
        Value::Array(items) => {
            for item in items {
                write_element(out, &name, item, depth);
            }
        }
        Value::Object(o) => {
            let mut attrs = String::new();
            let mut text = None;
            let mut children = Vec::new();
            for (k, v) in o {
                if let Some(a) = k.strip_prefix('@') {
                    attrs.push_str(&format!(
                        " {}=\"{}\"",
                        xml_name(a),
                        xml::escape(&scalar_text(v))
                    ));
                } else if k == "#text" {
                    text = Some(scalar_text(v));
                } else {
                    children.push((k, v));
                }
            }
            if children.is_empty() {
                match text {
                    Some(t) => out.push_str(&format!(
                        "{pad}<{name}{attrs}>{}</{name}>\n",
                        xml::escape(&t)
                    )),
                    None => out.push_str(&format!("{pad}<{name}{attrs}/>\n")),
                }
            } else {
                out.push_str(&format!("{pad}<{name}{attrs}>\n"));
                if let Some(t) = text {
                    out.push_str(&format!("{pad}  {}\n", xml::escape(&t)));
                }
                for (k, v) in children {
                    write_element(out, k, v, depth + 1);
                }
                out.push_str(&format!("{pad}</{name}>\n"));
            }
        }
        scalar => {
            let t = scalar_text(scalar);
            if t.is_empty() {
                out.push_str(&format!("{pad}<{name}/>\n"));
            } else {
                out.push_str(&format!("{pad}<{name}>{}</{name}>\n", xml::escape(&t)));
            }
        }
    }
}

fn write_xml(value: &Value) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    match value {
        // A single top-level key names the root element.
        Value::Object(o) if o.len() == 1 && !o.values().next().is_some_and(Value::is_array) => {
            let (k, v) = o.iter().next().expect("one");
            write_element(&mut out, k, v, 0);
        }
        Value::Array(items) => {
            out.push_str("<root>\n");
            for item in items {
                write_element(&mut out, "item", item, 1);
            }
            out.push_str("</root>\n");
        }
        other => write_element(&mut out, "root", other, 0),
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_yaml_roundtrips_through_json() {
        let yaml = "a:\n  b: [1, 2]\n  c: \"007\"\nd: true\n";
        let v = read(yaml.as_bytes(), "yaml").unwrap();
        assert_eq!(v["a"]["b"][1], 2);
        assert_eq!(v["a"]["c"], "007");
        let back = read(&write(&v, "yaml").unwrap(), "yaml").unwrap();
        assert_eq!(back, v);
    }

    #[test]
    fn yaml_quotes_ambiguous_strings() {
        let v: Value = serde_json::json!({"s": "true", "n": "1.0", "c": "a: b # c"});
        let yaml = String::from_utf8(write(&v, "yaml").unwrap()).unwrap();
        assert_eq!(read(yaml.as_bytes(), "yaml").unwrap(), v);
    }

    #[test]
    fn toml_arrays_of_tables() {
        let v: Value =
            serde_json::json!({"t": "x", "servers": [{"host": "a"}, {"host": "b"}], "n": null});
        let toml = String::from_utf8(write(&v, "toml").unwrap()).unwrap();
        assert!(toml.contains("[[servers]]"));
        let back = read(toml.as_bytes(), "toml").unwrap();
        assert_eq!(back["servers"][1]["host"], "b");
        assert!(back.get("n").is_none());
    }

    #[test]
    fn xml_json_roundtrip() {
        let x =
            r#"<config version="3"><title>Hi &amp; bye</title><tag>a</tag><tag>b</tag></config>"#;
        let v = read(x.as_bytes(), "xml").unwrap();
        assert_eq!(v["config"]["@version"], "3");
        assert_eq!(v["config"]["tag"][1], "b");
        let back = read(&write(&v, "xml").unwrap(), "xml").unwrap();
        assert_eq!(back, v);
    }
}
