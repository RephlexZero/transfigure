//! A small document model shared by every document format.
//!
//! Each input format has a reader that produces a [`Doc`], and each output
//! format has a writer that consumes one, so any document input converts to
//! any document output with the same structure preserved: headings,
//! paragraphs, emphasis, links, lists, quotes, code and tables.

mod docx;
mod html;
mod markdown;
mod odt;
mod pdf_in;
mod rtf;
mod text;

/// Inline formatting for a run of text.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub code: bool,
    pub strike: bool,
}

/// A run of text with one style. A `"\n"` span is a hard line break.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    pub text: String,
    pub style: Style,
    pub link: Option<String>,
}

impl Span {
    pub fn plain(text: impl Into<String>) -> Self {
        Span {
            text: text.into(),
            style: Style::default(),
            link: None,
        }
    }

    pub fn line_break() -> Self {
        Span::plain("\n")
    }

    pub fn is_break(&self) -> bool {
        self.text == "\n"
    }
}

pub type Inlines = Vec<Span>;

#[derive(Clone, Debug, PartialEq)]
pub enum Block {
    Heading(u8, Inlines),
    Para(Inlines),
    List {
        ordered: bool,
        start: u32,
        items: Vec<Vec<Block>>,
    },
    Quote(Vec<Block>),
    Code(String),
    /// First row is the header.
    Table(Vec<Vec<Inlines>>),
    Rule,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Doc {
    pub blocks: Vec<Block>,
}

impl Doc {
    /// Text of the first heading, used as a document title.
    pub fn title(&self) -> Option<String> {
        self.blocks.iter().find_map(|b| match b {
            Block::Heading(_, inl) => Some(plain_text(inl)),
            _ => None,
        })
    }
}

/// Concatenated text of some inlines, with line breaks as spaces.
pub fn plain_text(inlines: &[Span]) -> String {
    let mut s = String::new();
    for span in inlines {
        if span.is_break() {
            s.push(' ');
        } else {
            s.push_str(&span.text);
        }
    }
    s
}

/// Append text to an inline list, merging with the previous span when the
/// formatting matches so writers don't emit needless markup boundaries.
pub fn push_text(out: &mut Inlines, text: &str, style: Style, link: Option<&str>) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = out.last_mut()
        && !last.is_break()
        && text != "\n"
        && last.style == style
        && last.link.as_deref() == link
    {
        last.text.push_str(text);
        return;
    }
    out.push(Span {
        text: text.to_string(),
        style,
        link: link.map(str::to_string),
    });
}

/// Collapse runs of whitespace (as HTML/XML sources contain) and trim the ends
/// of a paragraph, keeping hard line breaks.
pub fn normalize_inlines(inlines: Inlines) -> Inlines {
    let mut out: Inlines = Vec::new();
    let mut prev_space = true;
    for mut span in inlines {
        if span.is_break() {
            if let Some(last) = out.last_mut() {
                let t = last.text.trim_end().to_string();
                last.text = t;
            }
            out.push(span);
            prev_space = true;
            continue;
        }
        if span.style.code {
            prev_space = span.text.ends_with(' ');
            out.push(span);
            continue;
        }
        let mut t = String::with_capacity(span.text.len());
        for c in span.text.chars() {
            if c.is_whitespace() && c != '\u{a0}' {
                if !prev_space {
                    t.push(' ');
                }
                prev_space = true;
            } else {
                t.push(c);
                prev_space = false;
            }
        }
        span.text = t;
        if !span.text.is_empty() {
            out.push(span);
        }
    }
    while out.last().is_some_and(|s| s.is_break()) {
        out.pop();
    }
    if let Some(last) = out.last_mut()
        && !last.style.code
    {
        let t = last.text.trim_end().to_string();
        last.text = t;
    }
    out.retain(|s| !s.text.is_empty());
    out
}

// ── Readers ─────────────────────────────────────────────

fn utf8(input: &[u8]) -> Result<&str, String> {
    let text = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    Ok(text.strip_prefix('\u{feff}').unwrap_or(text))
}

pub fn read(input: &[u8], from: &str) -> Result<Doc, String> {
    let mut doc = read_format(input, from)?;
    // Headings are bold by definition; explicit bold on them (common in word
    // processor output) would only add noise like `# **Title**`.
    for b in &mut doc.blocks {
        if let Block::Heading(_, inl) = b {
            for s in inl.iter_mut() {
                s.style.bold = false;
            }
            let merged = std::mem::take(inl);
            for s in merged {
                push_text(inl, &s.text, s.style, s.link.as_deref());
            }
        }
    }
    Ok(doc)
}

fn read_format(input: &[u8], from: &str) -> Result<Doc, String> {
    match from {
        "md" | "markdown" => Ok(markdown::read(utf8(input)?)),
        "html" | "htm" => Ok(html::read(&String::from_utf8_lossy(input))),
        "txt" | "text" => Ok(text::read(utf8(input)?)),
        "docx" => docx::read(input),
        "odt" => odt::read(input),
        "rtf" => rtf::read(input),
        "pdf" => pdf_in::read(input),
        _ => Err(format!("Unsupported document format: {from}")),
    }
}

// ── Writers ─────────────────────────────────────────────

pub fn write(doc: &Doc, to: &str) -> Result<Vec<u8>, String> {
    match to {
        "md" | "markdown" => Ok(markdown::write(doc).into_bytes()),
        "html" | "htm" => Ok(html::write(doc).into_bytes()),
        "txt" | "text" => Ok(text::write(doc).into_bytes()),
        "docx" => docx::write(doc),
        "pdf" => Ok(crate::pdf::render_doc(doc)),
        _ => Err(format!("Unsupported document output: {to}")),
    }
}

/// Convert between document formats. A few pairs take a more direct route
/// than the shared model because a dedicated library does better.
pub fn convert(input: &[u8], from: &str, to: &str) -> Result<Vec<u8>, String> {
    match (from, to) {
        // comrak renders every Markdown feature (footnotes, images, raw HTML).
        ("md" | "markdown", "html") => Ok(markdown::to_html(utf8(input)?).into_bytes()),
        // htmd handles arbitrary real-world HTML better than a round trip.
        ("html" | "htm", "md" | "markdown") => {
            Ok(html::to_markdown(&String::from_utf8_lossy(input)).into_bytes())
        }
        _ => write(&read(input, from)?, to),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_text_merges_same_style() {
        let mut v = Vec::new();
        push_text(&mut v, "a", Style::default(), None);
        push_text(&mut v, "b", Style::default(), None);
        push_text(
            &mut v,
            "c",
            Style {
                bold: true,
                ..Style::default()
            },
            None,
        );
        assert_eq!(v.len(), 2);
        assert_eq!(v[0].text, "ab");
    }

    #[test]
    fn normalize_collapses_whitespace() {
        let v = normalize_inlines(vec![Span::plain("  a \n  b  "), Span::plain(" c ")]);
        assert_eq!(plain_text(&v), "a b c");
    }
}
