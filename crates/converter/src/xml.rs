//! Minimal XML tree built on quick-xml, for the zipped-XML formats (DOCX,
//! ODT) and XML ↔ JSON. Names keep their prefix (`w:p`), which is how those
//! formats are written in practice.

use quick_xml::Reader;
use quick_xml::events::Event;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Element {
    pub name: String,
    pub attrs: Vec<(String, String)>,
    pub children: Vec<Node>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Node {
    Elem(Element),
    Text(String),
}

impl Element {
    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|n| match n {
            Node::Elem(e) => Some(e),
            Node::Text(_) => None,
        })
    }

    pub fn child(&self, name: &str) -> Option<&Element> {
        self.elements().find(|e| e.name == name)
    }

    /// Depth-first search for the first descendant with this name.
    pub fn find(&self, name: &str) -> Option<&Element> {
        for e in self.elements() {
            if e.name == name {
                return Some(e);
            }
            if let Some(found) = e.find(name) {
                return Some(found);
            }
        }
        None
    }

    /// All text content, concatenated.
    pub fn text(&self) -> String {
        let mut s = String::new();
        for n in &self.children {
            match n {
                Node::Text(t) => s.push_str(t),
                Node::Elem(e) => s.push_str(&e.text()),
            }
        }
        s
    }
}

fn entity(name: &str) -> Option<char> {
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{a0}',
        _ => return None,
    })
}

pub fn parse(xml: &str) -> Result<Element, String> {
    let mut reader = Reader::from_str(xml);
    let mut stack: Vec<Element> = vec![Element::default()];
    let err = |e: &dyn std::fmt::Display| format!("Invalid XML: {e}");

    let decoder = reader.decoder();
    let open = |e: &quick_xml::events::BytesStart| -> Result<Element, String> {
        let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
        let mut attrs = Vec::new();
        for a in e.attributes().with_checks(false) {
            let a = a.map_err(|e| err(&e))?;
            let key = String::from_utf8_lossy(a.key.as_ref()).into_owned();
            let value = a
                .decoded_and_normalized_value(quick_xml::XmlVersion::Implicit1_0, decoder)
                .map(|v| v.into_owned())
                .unwrap_or_else(|_| String::from_utf8_lossy(&a.value).into_owned());
            attrs.push((key, value));
        }
        Ok(Element {
            name,
            attrs,
            children: Vec::new(),
        })
    };

    let push_text = |stack: &mut Vec<Element>, text: &str| {
        let top = stack.last_mut().expect("root");
        if let Some(Node::Text(t)) = top.children.last_mut() {
            t.push_str(text);
        } else {
            top.children.push(Node::Text(text.to_string()));
        }
    };

    loop {
        match reader.read_event().map_err(|e| err(&e))? {
            Event::Start(e) => stack.push(open(&e)?),
            Event::Empty(e) => {
                let el = open(&e)?;
                stack
                    .last_mut()
                    .expect("root")
                    .children
                    .push(Node::Elem(el));
            }
            Event::End(_) => {
                if stack.len() > 1 {
                    let el = stack.pop().expect("checked");
                    stack
                        .last_mut()
                        .expect("root")
                        .children
                        .push(Node::Elem(el));
                }
            }
            Event::Text(t) => {
                let text = t.decode().map_err(|e| err(&e))?;
                push_text(&mut stack, &text);
            }
            Event::CData(t) => {
                let text = t.decode().map_err(|e| err(&e))?;
                push_text(&mut stack, &text);
            }
            Event::GeneralRef(r) => {
                let ch = match r.resolve_char_ref() {
                    Ok(Some(c)) => Some(c),
                    _ => entity(&r.decode().map_err(|e| err(&e))?),
                };
                if let Some(c) = ch {
                    push_text(&mut stack, c.encode_utf8(&mut [0; 4]));
                }
            }
            Event::Eof => break,
            _ => {}
        }
    }
    // Close anything left open by a truncated document.
    while stack.len() > 1 {
        let el = stack.pop().expect("checked");
        stack
            .last_mut()
            .expect("root")
            .children
            .push(Node::Elem(el));
    }
    let root = stack.pop().expect("root");
    root.elements()
        .next()
        .cloned()
        .ok_or_else(|| "XML has no root element".to_string())
}

pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(c),
        }
    }
    out
}

/// Read one file out of a ZIP container (DOCX, ODT, XLSX…) as text.
pub fn zip_entry(input: &[u8], path: &str) -> Result<Option<String>, String> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(input))
        .map_err(|e| format!("Not a valid ZIP-based document: {e}"))?;
    let Ok(mut file) = zip.by_name(path) else {
        return Ok(None);
    };
    let mut s = String::new();
    std::io::Read::read_to_string(&mut file, &mut s)
        .map_err(|e| format!("Failed to read {path}: {e}"))?;
    Ok(Some(s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_entities_and_attributes() {
        let root =
            parse(r#"<?xml version="1.0"?><a x="1 &amp; 2"><b>Caf&#233; &amp; co</b><c/></a>"#)
                .unwrap();
        assert_eq!(root.name, "a");
        assert_eq!(root.attr("x"), Some("1 & 2"));
        assert_eq!(root.child("b").unwrap().text(), "Café & co");
        assert!(root.child("c").is_some());
    }
}
