use comrak::nodes::{AstNode, ListType, NodeValue};
use comrak::{Arena, Options};

use super::{Block, Doc, Inlines, Span, Style, html, push_text};

fn options() -> Options<'static> {
    let mut o = Options::default();
    o.extension.table = true;
    o.extension.strikethrough = true;
    o.extension.autolink = true;
    o.extension.tasklist = true;
    o.extension.footnotes = true;
    // Converting your own file: raw HTML in Markdown should pass through, as
    // the CommonMark spec intends.
    o.render.unsafe_ = true;
    o
}

pub fn to_html(md: &str) -> String {
    let body = comrak::markdown_to_html(md, &options());
    let title = read(md).title();
    html::html_document(title.as_deref(), &body)
}

// ── Reader ──────────────────────────────────────────────

pub fn read(md: &str) -> Doc {
    let arena = Arena::new();
    let root = comrak::parse_document(&arena, md, &options());
    Doc {
        blocks: blocks(root),
    }
}

fn blocks<'a>(node: &'a AstNode<'a>) -> Vec<Block> {
    let mut out = Vec::new();
    for child in node.children() {
        let value = child.data.borrow().value.clone();
        match value {
            NodeValue::Paragraph => {
                let inl = inlines_of(child);
                if !inl.is_empty() {
                    out.push(Block::Para(inl));
                }
            }
            NodeValue::Heading(h) => out.push(Block::Heading(h.level, inlines_of(child))),
            NodeValue::List(l) => out.push(Block::List {
                ordered: l.list_type == ListType::Ordered,
                start: l.start.max(1) as u32,
                items: child.children().map(list_item).collect(),
            }),
            NodeValue::BlockQuote | NodeValue::MultilineBlockQuote(_) | NodeValue::Alert(_) => {
                out.push(Block::Quote(blocks(child)))
            }
            NodeValue::CodeBlock(c) => {
                out.push(Block::Code(c.literal.trim_end_matches('\n').to_string()))
            }
            NodeValue::ThematicBreak => out.push(Block::Rule),
            NodeValue::Table(_) => {
                let rows: Vec<Vec<Inlines>> = child
                    .children()
                    .map(|row| row.children().map(inlines_of).collect())
                    .collect();
                if !rows.is_empty() {
                    out.push(Block::Table(rows));
                }
            }
            NodeValue::HtmlBlock(h) => out.extend(html::read(&h.literal).blocks),
            _ => out.extend(blocks(child)),
        }
    }
    out
}

fn list_item<'a>(item: &'a AstNode<'a>) -> Vec<Block> {
    let mut b = blocks(item);
    if let NodeValue::TaskItem(mark) = item.data.borrow().value {
        let prefix = if mark.is_some() { "[x] " } else { "[ ] " };
        match b.first_mut() {
            Some(Block::Para(inl)) => inl.insert(0, Span::plain(prefix)),
            _ => b.insert(0, Block::Para(vec![Span::plain(prefix.trim_end())])),
        }
    }
    b
}

fn inlines_of<'a>(node: &'a AstNode<'a>) -> Inlines {
    let mut out = Vec::new();
    collect(node, Style::default(), None, &mut out);
    out
}

fn collect<'a>(node: &'a AstNode<'a>, style: Style, link: Option<&str>, out: &mut Inlines) {
    for child in node.children() {
        let value = child.data.borrow().value.clone();
        match value {
            NodeValue::Text(t) => push_text(out, &t, style, link),
            NodeValue::SoftBreak => push_text(out, " ", style, link),
            NodeValue::LineBreak => out.push(Span::line_break()),
            NodeValue::Code(c) => push_text(
                out,
                &c.literal,
                Style {
                    code: true,
                    ..style
                },
                link,
            ),
            NodeValue::Emph => collect(
                child,
                Style {
                    italic: true,
                    ..style
                },
                link,
                out,
            ),
            NodeValue::Strong => collect(
                child,
                Style {
                    bold: true,
                    ..style
                },
                link,
                out,
            ),
            NodeValue::Strikethrough => collect(
                child,
                Style {
                    strike: true,
                    ..style
                },
                link,
                out,
            ),
            NodeValue::Link(l) => collect(child, style, Some(&l.url), out),
            NodeValue::Image(_) => collect(child, style, link, out),
            NodeValue::FootnoteReference(r) => {
                push_text(out, &format!("[{}]", r.name), style, link)
            }
            NodeValue::HtmlInline(h) => {
                if h.to_ascii_lowercase().starts_with("<br") {
                    out.push(Span::line_break());
                }
            }
            _ => collect(child, style, link, out),
        }
    }
}

// ── Writer ──────────────────────────────────────────────

pub fn write(doc: &Doc) -> String {
    let mut s = render_blocks(&doc.blocks);
    while s.ends_with("\n\n") {
        s.pop();
    }
    s
}

fn render_blocks(blocks: &[Block]) -> String {
    let mut s = String::new();
    for b in blocks {
        match b {
            Block::Heading(level, inl) => {
                s.push_str(&"#".repeat((*level).clamp(1, 6) as usize));
                s.push(' ');
                s.push_str(&render_inlines(inl).replace('\n', " "));
                s.push_str("\n\n");
            }
            Block::Para(inl) => {
                s.push_str(&escape_line_start(&render_inlines(inl)));
                s.push_str("\n\n");
            }
            Block::List {
                ordered,
                start,
                items,
            } => {
                for (i, item) in items.iter().enumerate() {
                    let marker = if *ordered {
                        format!("{}. ", *start as usize + i)
                    } else {
                        "- ".to_string()
                    };
                    // A nested list follows its paragraph directly, keeping
                    // the list tight.
                    let mut body = String::new();
                    for (k, child) in item.iter().enumerate() {
                        let part = render_blocks(std::slice::from_ref(child));
                        body.push_str(part.trim_end());
                        if let Some(next) = item.get(k + 1) {
                            body.push_str(if matches!(next, Block::List { .. }) {
                                "\n"
                            } else {
                                "\n\n"
                            });
                        }
                    }
                    let body = body.trim_end();
                    let pad = " ".repeat(marker.len());
                    s.push_str(&marker);
                    for (j, line) in body.lines().enumerate() {
                        if j > 0 {
                            s.push('\n');
                            if !line.is_empty() {
                                s.push_str(&pad);
                            }
                        }
                        s.push_str(line);
                    }
                    s.push('\n');
                }
                s.push('\n');
            }
            Block::Quote(inner) => {
                for line in render_blocks(inner).trim_end().lines() {
                    if line.is_empty() {
                        s.push_str(">\n");
                    } else {
                        s.push_str("> ");
                        s.push_str(line);
                        s.push('\n');
                    }
                }
                s.push('\n');
            }
            Block::Code(code) => {
                let mut fence = "```".to_string();
                while code.contains(&fence) {
                    fence.push('`');
                }
                s.push_str(&format!("{fence}\n{code}\n{fence}\n\n"));
            }
            Block::Table(rows) => {
                s.push_str(&render_table(rows));
                s.push('\n');
            }
            Block::Rule => s.push_str("---\n\n"),
        }
    }
    s
}

fn render_table(rows: &[Vec<Inlines>]) -> String {
    let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
    if cols == 0 {
        return String::new();
    }
    let cell = |row: &Vec<Inlines>, i: usize| {
        row.get(i)
            .map(|c| render_inlines(c).replace('\n', " ").replace('|', "\\|"))
            .unwrap_or_default()
    };
    let mut s = String::new();
    for (r, row) in rows.iter().enumerate() {
        s.push('|');
        for i in 0..cols {
            s.push(' ');
            s.push_str(&cell(row, i));
            s.push_str(" |");
        }
        s.push('\n');
        if r == 0 {
            s.push('|');
            s.push_str(&" --- |".repeat(cols));
            s.push('\n');
        }
    }
    s
}

/// Markdown for a list of inline spans.
pub fn render_inlines(inl: &[Span]) -> String {
    let mut s = String::new();
    let mut i = 0;
    while i < inl.len() {
        // Group consecutive spans sharing a link so the link wraps them all.
        if let Some(url) = &inl[i].link {
            let mut j = i;
            while j < inl.len() && inl[j].link.as_ref() == Some(url) {
                j += 1;
            }
            let label: String = inl[i..j].iter().map(render_span).collect();
            if label.is_empty() || &label == url {
                s.push_str(&format!("<{url}>"));
            } else {
                s.push_str(&format!("[{label}]({})", url.replace(' ', "%20")));
            }
            i = j;
        } else {
            s.push_str(&render_span(&inl[i]));
            i += 1;
        }
    }
    s
}

fn render_span(span: &Span) -> String {
    if span.is_break() {
        return "  \n".into();
    }
    let st = span.style;
    if st.code {
        let ticks = if span.text.contains('`') { "``" } else { "`" };
        let pad = if span.text.starts_with('`') || span.text.ends_with('`') {
            " "
        } else {
            ""
        };
        return format!("{ticks}{pad}{}{pad}{ticks}", span.text);
    }
    let text = escape(&span.text);
    let mut marks = String::new();
    if st.bold {
        marks.push_str("**");
    }
    if st.italic {
        marks.push('*');
    }
    if st.strike {
        marks.push_str("~~");
    }
    if marks.is_empty() {
        return text;
    }
    // Emphasis markers must hug the text, so keep outer spaces outside them.
    let core = text.trim();
    if core.is_empty() {
        return text;
    }
    let lead = &text[..text.len() - text.trim_start().len()];
    let trail = &text[text.trim_end().len()..];
    let close: String = marks.chars().rev().collect();
    format!("{lead}{marks}{core}{close}{trail}")
}

fn escape(text: &str) -> String {
    let mut s = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '\\' | '*' | '_' | '`' | '[' | ']' | '<' | '~') {
            s.push('\\');
        }
        s.push(c);
    }
    s
}

/// Stop a paragraph that happens to start like a heading or list item from
/// being read back as one.
fn escape_line_start(text: &str) -> String {
    let t = text.trim_start();
    let starts_structural = t.starts_with('#')
        || t.starts_with("- ")
        || t.starts_with("+ ")
        || t.starts_with('>')
        || t.split_once(". ")
            .is_some_and(|(n, _)| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()));
    if !starts_structural {
        return text.to_string();
    }
    if let Some(pos) = t.find(". ").filter(|_| t.as_bytes()[0].is_ascii_digit()) {
        format!("{}\\{}", &t[..pos], &t[pos..])
    } else {
        format!("\\{t}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_structure() {
        let md = "# Title\n\nSome **bold** and *it* and `code` [link](https://x.y).\n\n- a\n- b\n  1. c\n\n> quote\n\n| h | i |\n| --- | --- |\n| 1 | 2 |\n\n```\nx < y\n```\n";
        let doc = read(md);
        assert!(matches!(doc.blocks[0], Block::Heading(1, _)));
        let out = write(&doc);
        assert_eq!(read(&out), doc, "markdown:\n{out}");
    }

    #[test]
    fn escapes_literal_markup() {
        let doc = Doc {
            blocks: vec![Block::Para(vec![Span::plain(
                "2 * 3 = 6 and 1. not a list",
            )])],
        };
        let out = write(&doc);
        assert_eq!(read(&out), doc, "markdown:\n{out}");
    }

    #[test]
    fn emphasis_keeps_spaces_outside_markers() {
        let span = Span {
            text: " bold ".into(),
            style: Style {
                bold: true,
                ..Style::default()
            },
            link: None,
        };
        assert_eq!(render_span(&span), " **bold** ");
    }
}
