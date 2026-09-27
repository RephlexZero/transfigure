//! OpenDocument Text (ODT) reading.

use std::collections::HashMap;

use super::{Block, Doc, Inlines, Span, Style, normalize_inlines, push_text};
use crate::xml::{self, Element, Node};

#[derive(Default)]
struct StyleDef {
    parent: Option<String>,
    bold: Option<bool>,
    italic: Option<bool>,
    mono: bool,
    list_style: Option<String>,
}

#[derive(Default)]
struct Ctx {
    styles: HashMap<String, StyleDef>,
    /// list style → per-level "is numbered"
    lists: HashMap<String, Vec<bool>>,
}

pub fn read(input: &[u8]) -> Result<Doc, String> {
    let content = xml::zip_entry(input, "content.xml")?
        .ok_or_else(|| "content.xml not found — is this an OpenDocument file?".to_string())?;
    let mut ctx = Ctx::default();
    if let Some(styles) = xml::zip_entry(input, "styles.xml")? {
        collect_styles(&xml::parse(&styles)?, &mut ctx);
    }
    let root = xml::parse(&content)?;
    collect_styles(&root, &mut ctx);
    let text = root
        .find("office:text")
        .ok_or_else(|| "ODT has no text body".to_string())?;
    Ok(Doc {
        blocks: blocks(text, &ctx, None, 0),
    })
}

fn collect_styles(root: &Element, ctx: &mut Ctx) {
    for section in root
        .elements()
        .filter(|e| matches!(e.name.as_str(), "office:styles" | "office:automatic-styles"))
    {
        for s in section.elements() {
            match s.name.as_str() {
                "style:style" => {
                    let Some(name) = s.attr("style:name") else {
                        continue;
                    };
                    let props = s.child("style:text-properties");
                    let weight = props.and_then(|p| p.attr("fo:font-weight"));
                    let font = props
                        .and_then(|p| p.attr("style:font-name"))
                        .unwrap_or("")
                        .to_lowercase();
                    ctx.styles.insert(
                        name.to_string(),
                        StyleDef {
                            parent: s.attr("style:parent-style-name").map(str::to_string),
                            bold: weight
                                .map(|w| w == "bold" || w.parse::<u16>().is_ok_and(|n| n >= 600)),
                            italic: props
                                .and_then(|p| p.attr("fo:font-style"))
                                .map(|v| v == "italic" || v == "oblique"),
                            mono: font.contains("mono") || font.contains("courier"),
                            list_style: s.attr("style:list-style-name").map(str::to_string),
                        },
                    );
                }
                "text:list-style" => {
                    let Some(name) = s.attr("style:name") else {
                        continue;
                    };
                    let levels = s
                        .elements()
                        .map(|l| l.name == "text:list-level-style-number")
                        .collect();
                    ctx.lists.insert(name.to_string(), levels);
                }
                _ => {}
            }
        }
    }
}

/// Style name and its ancestors, nearest first.
fn chain<'a>(ctx: &'a Ctx, name: &'a str) -> Vec<&'a str> {
    let mut out = vec![name];
    let mut cur = name;
    while let Some(parent) = ctx.styles.get(cur).and_then(|s| s.parent.as_deref()) {
        if out.contains(&parent) || out.len() > 16 {
            break;
        }
        out.push(parent);
        cur = parent;
    }
    out
}

fn heading_level(ctx: &Ctx, style: &str) -> Option<u8> {
    chain(ctx, style).iter().find_map(|n| {
        let n = n.replace("_20_", " ").to_lowercase();
        if let Some(l) = n.strip_prefix("heading ") {
            l.trim().parse::<u8>().ok()
        } else if n == "title" {
            Some(1)
        } else if n == "subtitle" {
            Some(2)
        } else {
            None
        }
    })
}

fn style_named(ctx: &Ctx, style: &str, names: &[&str]) -> bool {
    chain(ctx, style).iter().any(|n| {
        let n = n.replace("_20_", " ").to_lowercase();
        names.contains(&n.as_str())
    })
}

fn span_style(ctx: &Ctx, name: &str, mut st: Style) -> Style {
    let ch = chain(ctx, name);
    if let Some(b) = ch
        .iter()
        .find_map(|n| ctx.styles.get(*n).and_then(|s| s.bold))
    {
        st.bold = b;
    }
    if let Some(i) = ch
        .iter()
        .find_map(|n| ctx.styles.get(*n).and_then(|s| s.italic))
    {
        st.italic = i;
    }
    if ch
        .iter()
        .any(|n| ctx.styles.get(*n).is_some_and(|s| s.mono))
    {
        st.code = true;
    }
    for n in &ch {
        match n.replace("_20_", " ").to_lowercase().as_str() {
            "strong emphasis" | "strong" => st.bold = true,
            "emphasis" => st.italic = true,
            "source text" | "teletype" | "example" => st.code = true,
            _ => {}
        }
    }
    st
}

fn blocks(parent: &Element, ctx: &Ctx, list_style: Option<&str>, depth: usize) -> Vec<Block> {
    let mut out = Vec::new();
    for el in parent.elements() {
        match el.name.as_str() {
            "text:h" => {
                let level = el
                    .attr("text:outline-level")
                    .and_then(|v| v.parse::<u8>().ok())
                    .unwrap_or(1);
                let inl = normalize_inlines(inlines(el, ctx, Style::default()));
                if !inl.is_empty() {
                    out.push(Block::Heading(level.clamp(1, 6), inl));
                }
            }
            "text:p" => {
                let style = el.attr("text:style-name").unwrap_or("");
                if style_named(ctx, style, &["preformatted text", "source text", "code"]) {
                    let text = raw_text(el);
                    if let Some(Block::Code(prev)) = out.last_mut() {
                        prev.push('\n');
                        prev.push_str(&text);
                    } else {
                        out.push(Block::Code(text));
                    }
                    continue;
                }
                let base = span_style(ctx, style, Style::default());
                let inl = normalize_inlines(inlines(el, ctx, base));
                if inl.is_empty() {
                    continue;
                }
                if let Some(level) = heading_level(ctx, style) {
                    out.push(Block::Heading(level.clamp(1, 6), inl));
                } else if style_named(ctx, style, &["quotations", "quote"]) {
                    out.push(Block::Quote(vec![Block::Para(inl)]));
                } else {
                    out.push(Block::Para(inl));
                }
            }
            "text:list" => {
                let ls = el.attr("text:style-name").or(list_style);
                let ordered = ls
                    .and_then(|n| ctx.lists.get(n))
                    .and_then(|levels| levels.get(depth.min(levels.len().saturating_sub(1))))
                    .copied()
                    .or_else(|| list_from_paragraphs(el, ctx))
                    .unwrap_or(false);
                let items: Vec<Vec<Block>> = el
                    .elements()
                    .filter(|e| matches!(e.name.as_str(), "text:list-item" | "text:list-header"))
                    .map(|item| blocks(item, ctx, ls, depth + 1))
                    .filter(|b| !b.is_empty())
                    .collect();
                if !items.is_empty() {
                    out.push(Block::List {
                        ordered,
                        start: 1,
                        items,
                    });
                }
            }
            "table:table" => out.push(table(el, ctx)),
            "text:section" | "text:index-body" | "table:table-header-rows" => {
                out.extend(blocks(el, ctx, list_style, depth))
            }
            _ => {}
        }
    }
    out
}

/// Some producers attach the list style to the paragraphs instead.
fn list_from_paragraphs(list: &Element, ctx: &Ctx) -> Option<bool> {
    let p = list.find("text:p")?;
    let style = ctx.styles.get(p.attr("text:style-name")?)?;
    ctx.lists
        .get(style.list_style.as_deref()?)
        .and_then(|l| l.first().copied())
}

fn inlines(el: &Element, ctx: &Ctx, style: Style) -> Inlines {
    let mut out = Vec::new();
    collect(el, ctx, style, None, &mut out);
    out
}

fn collect(el: &Element, ctx: &Ctx, style: Style, link: Option<&str>, out: &mut Inlines) {
    for n in &el.children {
        match n {
            Node::Text(t) => push_text(out, t, style, link),
            Node::Elem(c) => match c.name.as_str() {
                "text:span" => {
                    let st = c
                        .attr("text:style-name")
                        .map_or(style, |s| span_style(ctx, s, style));
                    collect(c, ctx, st, link, out);
                }
                "text:a" => collect(c, ctx, style, c.attr("xlink:href").or(link), out),
                "text:s" => {
                    let n = c
                        .attr("text:c")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(1usize);
                    // Keep explicit spaces through whitespace normalisation.
                    push_text(out, &"\u{a0}".repeat(n.min(64)), style, link);
                }
                "text:tab" => push_text(out, "\t", style, link),
                "text:line-break" => out.push(Span::line_break()),
                "text:note" | "office:annotation" | "text:bookmark-ref" => {}
                _ => collect(c, ctx, style, link, out),
            },
        }
    }
}

/// Text of a preformatted paragraph with explicit spaces restored.
fn raw_text(el: &Element) -> String {
    let mut s = String::new();
    for n in &el.children {
        match n {
            Node::Text(t) => s.push_str(t),
            Node::Elem(c) => match c.name.as_str() {
                "text:s" => {
                    let n = c
                        .attr("text:c")
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(1usize);
                    s.push_str(&" ".repeat(n.min(256)));
                }
                "text:tab" => s.push('\t'),
                "text:line-break" => s.push('\n'),
                _ => s.push_str(&raw_text(c)),
            },
        }
    }
    s
}

fn table(tbl: &Element, ctx: &Ctx) -> Block {
    let mut rows = Vec::new();
    let mut row_elems = Vec::new();
    for e in tbl.elements() {
        match e.name.as_str() {
            "table:table-row" => row_elems.push(e),
            "table:table-header-rows" | "table:table-rows" => {
                row_elems.extend(e.elements().filter(|r| r.name == "table:table-row"))
            }
            _ => {}
        }
    }
    for tr in row_elems {
        let mut row = Vec::new();
        for tc in tr.elements().filter(|e| e.name == "table:table-cell") {
            let mut cell: Inlines = Vec::new();
            for p in tc
                .elements()
                .filter(|e| matches!(e.name.as_str(), "text:p" | "text:h"))
            {
                let inl = normalize_inlines(inlines(p, ctx, Style::default()));
                if inl.is_empty() {
                    continue;
                }
                if !cell.is_empty() {
                    cell.push(Span::line_break());
                }
                cell.extend(inl);
            }
            let repeat = tc
                .attr("table:number-columns-repeated")
                .and_then(|v| v.parse().ok())
                .unwrap_or(1usize);
            // Spreadsheet-style trailing repeats can be huge; they are empty.
            for _ in 0..repeat.min(64) {
                row.push(cell.clone());
            }
        }
        while row.last().is_some_and(|c: &Inlines| c.is_empty()) {
            row.pop();
        }
        rows.push(row);
    }
    Block::Table(rows)
}
