use base64::Engine;

pub fn markdown_to_html(input: &[u8]) -> Result<Vec<u8>, String> {
    let md = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    let html = comrak::markdown_to_html(md, &comrak::Options::default());
    Ok(html.into_bytes())
}

pub fn markdown_to_text(input: &[u8]) -> Result<Vec<u8>, String> {
    let md = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    let html = comrak::markdown_to_html(md, &comrak::Options::default());
    let text = strip_html_tags(&html);
    Ok(text.into_bytes())
}

pub fn html_to_markdown(input: &[u8]) -> Result<Vec<u8>, String> {
    let html = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    let md = html_to_md(html);
    Ok(md.into_bytes())
}

pub fn base64_encode(input: &[u8]) -> Result<Vec<u8>, String> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(input);
    Ok(encoded.into_bytes())
}

pub fn base64_decode(input: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    base64::engine::general_purpose::STANDARD
        .decode(text.trim())
        .map_err(|e| format!("Invalid base64: {e}"))
}

pub fn json_to_yaml(input: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {e}"))?;
    let yaml = json_value_to_yaml(&value, 0);
    Ok(yaml.into_bytes())
}

pub fn yaml_to_json(input: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    let value = simple_yaml_parse(text)?;
    serde_json::to_string_pretty(&value)
        .map(|s| s.into_bytes())
        .map_err(|e| format!("JSON serialization error: {e}"))
}

pub fn toml_to_json(input: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    let value = simple_toml_parse(text)?;
    serde_json::to_string_pretty(&value)
        .map(|s| s.into_bytes())
        .map_err(|e| format!("JSON serialization error: {e}"))
}

pub fn json_to_toml(input: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {e}"))?;
    let toml = json_value_to_toml(&value, "");
    Ok(toml.into_bytes())
}

// ── Helpers ──────────────────────────────────────────────

fn strip_html_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    result
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .trim()
        .to_string()
}

fn html_to_md(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let mut tag = String::new();
    let mut chars = html.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            in_tag = true;
            tag.clear();
        } else if ch == '>' && in_tag {
            in_tag = false;
            let t = tag.trim().to_lowercase();
            match t.as_str() {
                "h1" => result.push_str("# "),
                "h2" => result.push_str("## "),
                "h3" => result.push_str("### "),
                "/h1" | "/h2" | "/h3" | "/h4" | "/h5" | "/h6" => {
                    result.push_str("\n\n");
                }
                "/p" => result.push_str("\n\n"),
                "br" | "br/" | "br /" => result.push('\n'),
                "strong" | "b" | "/strong" | "/b" => result.push_str("**"),
                "em" | "i" | "/em" | "/i" => result.push('*'),
                "code" | "/code" => result.push('`'),
                "li" => result.push_str("- "),
                "/li" => result.push('\n'),
                "hr" | "hr/" => result.push_str("\n---\n"),
                _ => {}
            }
        } else if in_tag {
            tag.push(ch);
        } else if ch == '&' {
            let mut entity = String::from("&");
            for next in chars.by_ref() {
                entity.push(next);
                if next == ';' {
                    break;
                }
            }
            match entity.as_str() {
                "&amp;" => result.push('&'),
                "&lt;" => result.push('<'),
                "&gt;" => result.push('>'),
                "&quot;" => result.push('"'),
                "&#39;" => result.push('\''),
                _ => result.push_str(&entity),
            }
        } else {
            result.push(ch);
        }
    }
    result.trim().to_string()
}

fn json_value_to_yaml(value: &serde_json::Value, indent: usize) -> String {
    let prefix = "  ".repeat(indent);
    match value {
        serde_json::Value::Null => "null\n".into(),
        serde_json::Value::Bool(b) => format!("{b}\n"),
        serde_json::Value::Number(n) => format!("{n}\n"),
        serde_json::Value::String(s) => format!("{s}\n"),
        serde_json::Value::Array(arr) => {
            let mut out = String::from("\n");
            for item in arr {
                out.push_str(&format!("{prefix}- "));
                out.push_str(json_value_to_yaml(item, indent + 1).trim_start());
            }
            out
        }
        serde_json::Value::Object(map) => {
            let mut out = if indent > 0 {
                "\n".into()
            } else {
                String::new()
            };
            for (key, val) in map {
                out.push_str(&format!("{prefix}{key}: "));
                out.push_str(json_value_to_yaml(val, indent + 1).trim_start());
            }
            out
        }
    }
}

fn json_value_to_toml(value: &serde_json::Value, prefix: &str) -> String {
    let mut out = String::new();
    if let serde_json::Value::Object(map) = value {
        for (key, val) in map {
            match val {
                serde_json::Value::Object(_) => {}
                serde_json::Value::Array(arr) => {
                    let items: Vec<String> = arr.iter().map(toml_scalar).collect();
                    out.push_str(&format!("{key} = [{}]\n", items.join(", ")));
                }
                _ => out.push_str(&format!("{key} = {}\n", toml_scalar(val))),
            }
        }
        for (key, val) in map {
            if let serde_json::Value::Object(_) = val {
                let section = if prefix.is_empty() {
                    key.clone()
                } else {
                    format!("{prefix}.{key}")
                };
                out.push_str(&format!("\n[{section}]\n"));
                out.push_str(&json_value_to_toml(val, &section));
            }
        }
    }
    out
}

fn toml_scalar(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::String(s) => format!("\"{s}\""),
        serde_json::Value::Number(n) => format!("{n}"),
        serde_json::Value::Bool(b) => format!("{b}"),
        serde_json::Value::Null => "\"\"".into(),
        _ => format!("\"{val}\""),
    }
}

fn simple_yaml_parse(text: &str) -> Result<serde_json::Value, String> {
    let mut map = serde_json::Map::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line == "---" {
            continue;
        }
        if let Some((key, val)) = line.split_once(':') {
            map.insert(key.trim().to_string(), parse_yaml_value(val.trim()));
        }
    }
    Ok(serde_json::Value::Object(map))
}

fn parse_yaml_value(s: &str) -> serde_json::Value {
    match s {
        "" | "null" | "~" => serde_json::Value::Null,
        "true" => serde_json::Value::Bool(true),
        "false" => serde_json::Value::Bool(false),
        _ => {
            if let Ok(n) = s.parse::<i64>() {
                serde_json::Value::Number(n.into())
            } else if let Ok(n) = s.parse::<f64>() {
                serde_json::Number::from_f64(n)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(s.to_string()))
            } else {
                serde_json::Value::String(s.trim_matches('"').trim_matches('\'').to_string())
            }
        }
    }
}

fn simple_toml_parse(text: &str) -> Result<serde_json::Value, String> {
    let mut root = serde_json::Map::new();
    let mut current_section: Option<String> = None;

    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            current_section = Some(line[1..line.len() - 1].trim().to_string());
            continue;
        }
        if let Some((key, val)) = line.split_once('=') {
            let parsed = parse_toml_value(val.trim());
            if let Some(ref section) = current_section {
                let table = root
                    .entry(section.clone())
                    .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
                if let serde_json::Value::Object(m) = table {
                    m.insert(key.trim().to_string(), parsed);
                }
            } else {
                root.insert(key.trim().to_string(), parsed);
            }
        }
    }
    Ok(serde_json::Value::Object(root))
}

fn parse_toml_value(s: &str) -> serde_json::Value {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        serde_json::Value::String(s[1..s.len() - 1].to_string())
    } else if s == "true" {
        serde_json::Value::Bool(true)
    } else if s == "false" {
        serde_json::Value::Bool(false)
    } else if let Ok(n) = s.parse::<i64>() {
        serde_json::Value::Number(n.into())
    } else if let Ok(n) = s.parse::<f64>() {
        serde_json::Number::from_f64(n)
            .map(serde_json::Value::Number)
            .unwrap_or(serde_json::Value::String(s.to_string()))
    } else {
        serde_json::Value::String(s.to_string())
    }
}
