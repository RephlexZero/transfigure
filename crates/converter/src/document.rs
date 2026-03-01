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

// ── DOCX (Office Open XML) ───────────────────────────────

/// Extract plain text from a DOCX file.
///
/// DOCX files are ZIP archives containing XML. We read `word/document.xml`
/// and extract the text content of all `<w:t>` elements, inserting paragraph
/// breaks at `<w:p>` boundaries. No C libraries required — uses the `zip` crate
/// already in our dependency tree.
pub fn docx_to_text(input: &[u8]) -> Result<Vec<u8>, String> {
    let xml = read_docx_document_xml(input)?;
    let text = docx_xml_to_text(&xml);
    Ok(text.into_bytes())
}

/// Convert a DOCX file to a minimal HTML document.
///
/// Paragraphs (`<w:p>`) become `<p>` tags; bold runs (`<w:b/>` inside `<w:rPr>`)
/// become `<strong>`; italic (`<w:i/>`) become `<em>`. Headings styled as
/// `Heading1`–`Heading6` via `<w:pStyle w:val="Heading1">` become `<h1>`–`<h6>`.
pub fn docx_to_html(input: &[u8]) -> Result<Vec<u8>, String> {
    let xml = read_docx_document_xml(input)?;
    let html = docx_xml_to_html(&xml);
    Ok(html.into_bytes())
}

/// Read `word/document.xml` from a DOCX (ZIP) byte slice.
fn read_docx_document_xml(input: &[u8]) -> Result<String, String> {
    let cursor = std::io::Cursor::new(input);
    let mut zip =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to open DOCX as ZIP: {e}"))?;

    // Normalise path — some tools use "word/document.xml", others the same.
    let names: Vec<String> = (0..zip.len())
        .filter_map(|i| zip.by_index(i).ok().map(|f| f.name().to_lowercase()))
        .collect();

    let idx = names
        .iter()
        .position(|n| n == "word/document.xml")
        .ok_or_else(|| "word/document.xml not found in DOCX archive".to_string())?;

    let mut file = zip
        .by_index(idx)
        .map_err(|e| format!("ZIP read error: {e}"))?;
    let mut xml = String::new();
    std::io::Read::read_to_string(&mut file, &mut xml)
        .map_err(|e| format!("Failed to read document.xml: {e}"))?;
    Ok(xml)
}

/// Walk the XML from `word/document.xml` and produce plain text.
///
/// Strategy:
/// - `<w:p …>` → start new paragraph (emit `\n\n` before non-first paragraphs)
/// - `<w:t …>text</w:t>` → append text content
/// - `<w:br/>` → `\n`
fn docx_xml_to_text(xml: &str) -> String {
    let mut out = String::new();
    let mut in_t = false;
    let mut para_count = 0u32;
    let bytes = xml.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            // Collect full tag
            let start = i;
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1; // consume '>'
            }
            let tag = &xml[start..i];

            if tag.starts_with("<w:p ") || tag == "<w:p>" {
                if para_count > 0 {
                    out.push_str("\n\n");
                }
                para_count += 1;
                in_t = false;
            } else if tag.starts_with("<w:t") {
                in_t = true;
            } else if tag == "</w:t>" {
                in_t = false;
            } else if tag.starts_with("<w:br") {
                out.push('\n');
            }
        } else if in_t {
            // Collect text content until next '<'
            let start = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let text = &xml[start..i];
            // Decode minimal XML entities
            out.push_str(&decode_xml_entities(text));
        } else {
            i += 1;
        }
    }

    out.trim().to_string()
}

/// Walk the XML from `word/document.xml` and produce HTML.
fn docx_xml_to_html(xml: &str) -> String {
    let mut out = String::from("<html><body>\n");
    let mut in_t = false;
    let mut bold = false;
    let mut italic = false;
    let mut in_rpr = false;
    let mut heading_level: Option<u8> = None;
    let bytes = xml.as_bytes();
    let mut i = 0;
    let mut para_open = false;

    while i < bytes.len() {
        if bytes[i] == b'<' {
            let start = i;
            while i < bytes.len() && bytes[i] != b'>' {
                i += 1;
            }
            if i < bytes.len() {
                i += 1;
            }
            let tag = &xml[start..i];
            let tag_lower = tag.to_lowercase();

            if tag_lower.starts_with("<w:p ") || tag_lower == "<w:p>" {
                heading_level = None;
                bold = false;
                italic = false;
                in_rpr = false;
            } else if tag_lower.starts_with("<w:pstyle ") {
                // e.g. <w:pStyle w:val="Heading1">
                if let Some(val) = extract_attr(tag, "w:val") {
                    heading_level = match val.to_lowercase().as_str() {
                        "heading1" | "heading 1" => Some(1),
                        "heading2" | "heading 2" => Some(2),
                        "heading3" | "heading 3" => Some(3),
                        "heading4" | "heading 4" => Some(4),
                        "heading5" | "heading 5" => Some(5),
                        "heading6" | "heading 6" => Some(6),
                        _ => None,
                    };
                }
            } else if tag_lower == "<w:rpr>" {
                in_rpr = true;
                bold = false;
                italic = false;
            } else if tag_lower == "</w:rpr>" {
                in_rpr = false;
            } else if (tag_lower == "<w:b/>" || tag_lower == "<w:b />") && in_rpr {
                bold = true;
            } else if (tag_lower == "<w:i/>" || tag_lower == "<w:i />") && in_rpr {
                italic = true;
            } else if tag_lower.starts_with("<w:t") && !tag_lower.starts_with("<w:tbl") {
                if !para_open {
                    if let Some(h) = heading_level {
                        out.push_str(&format!("<h{h}>"));
                    } else {
                        out.push_str("<p>");
                    }
                    para_open = true;
                }
                if bold {
                    out.push_str("<strong>");
                }
                if italic {
                    out.push_str("<em>");
                }
                in_t = true;
            } else if tag_lower == "</w:t>" {
                if italic {
                    out.push_str("</em>");
                }
                if bold {
                    out.push_str("</strong>");
                }
                in_t = false;
            } else if tag_lower == "</w:p>" {
                if para_open {
                    if let Some(h) = heading_level {
                        out.push_str(&format!("</h{h}>\n"));
                    } else {
                        out.push_str("</p>\n");
                    }
                    para_open = false;
                }
                heading_level = None;
            } else if tag_lower.starts_with("<w:br") {
                out.push_str("<br/>");
            }
        } else if in_t {
            let start = i;
            while i < bytes.len() && bytes[i] != b'<' {
                i += 1;
            }
            let text = &xml[start..i];
            out.push_str(&html_escape(&decode_xml_entities(text)));
        } else {
            i += 1;
        }
    }

    out.push_str("</body></html>");
    out
}

fn extract_attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    // Finds   attr="value"  or  attr='value'  in a tag string
    let pat = format!("{attr}=");
    let pos = tag.find(pat.as_str())?;
    let rest = &tag[pos + pat.len()..];
    let quote = rest.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let inner = &rest[1..];
    let end = inner.find(quote)?;
    Some(&inner[..end])
}

fn decode_xml_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

// ── RTF ─────────────────────────────────────────────────

/// Strip RTF markup and return plain text.
///
/// Uses the `rtf-parser` crate (pure Rust, WASM-safe) to tokenise the RTF
/// stream, then collects all plain text tokens.
pub fn rtf_to_text(input: &[u8]) -> Result<Vec<u8>, String> {
    let rtf_str = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8 in RTF: {e}"))?;

    use rtf_parser::lexer::Lexer;
    use rtf_parser::tokens::Token;

    let tokens = Lexer::scan(rtf_str).map_err(|e| format!("RTF parse error: {e:?}"))?;

    let mut out = String::new();
    for token in &tokens {
        if let Token::PlainText(text) = token {
            out.push_str(text);
        }
    }

    Ok(out.trim().to_owned().into_bytes())
}
