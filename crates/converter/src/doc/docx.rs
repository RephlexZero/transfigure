//! DOCX (Office Open XML) reading and writing.

use std::collections::HashMap;
use std::io::Cursor;

use super::{Block, Doc, Inlines, Span, Style, normalize_inlines, push_text};
use crate::xml::{self, Element};

// ── Reader ──────────────────────────────────────────────

#[derive(Default)]
struct StyleInfo {
    name: String,
    bold: bool,
    italic: bool,
    outline: Option<u8>,
}

/// ilvl → (ordered, start)
type Levels = HashMap<u8, (bool, u32)>;

struct Ctx {
    rels: HashMap<String, String>,
    styles: HashMap<String, StyleInfo>,
    /// numId → its levels
    numbering: HashMap<String, Levels>,
}

/// A paragraph before list grouping.
enum Item {
    Block(Block),
    ListPara {
        level: u8,
        ordered: bool,
        start: u32,
        inlines: Inlines,
    },
}

pub fn read(input: &[u8]) -> Result<Doc, String> {
    let document = xml::zip_entry(input, "word/document.xml")?
        .ok_or_else(|| "word/document.xml not found — is this a Word document?".to_string())?;
    let ctx = Ctx {
        rels: read_rels(input)?,
        styles: read_styles(input)?,
        numbering: read_numbering(input)?,
    };
    let root = xml::parse(&document)?;
    let body = root
        .find("w:body")
        .ok_or_else(|| "DOCX has no document body".to_string())?;
    let mut items = Vec::new();
    body_items(body, &ctx, &mut items);
    Ok(Doc {
        blocks: group_lists(items),
    })
}

fn read_rels(input: &[u8]) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    if let Some(x) = xml::zip_entry(input, "word/_rels/document.xml.rels")? {
        let root = xml::parse(&x)?;
        for r in root.elements() {
            if let (Some(id), Some(target)) = (r.attr("Id"), r.attr("Target")) {
                map.insert(id.to_string(), target.to_string());
            }
        }
    }
    Ok(map)
}

fn on(el: Option<&Element>) -> bool {
    el.is_some_and(|e| !matches!(e.attr("w:val"), Some("0" | "false" | "none" | "off")))
}

fn read_styles(input: &[u8]) -> Result<HashMap<String, StyleInfo>, String> {
    let mut map = HashMap::new();
    if let Some(x) = xml::zip_entry(input, "word/styles.xml")? {
        let root = xml::parse(&x)?;
        for s in root.elements().filter(|e| e.name == "w:style") {
            let Some(id) = s.attr("w:styleId") else {
                continue;
            };
            let name = s
                .child("w:name")
                .and_then(|n| n.attr("w:val"))
                .unwrap_or(id)
                .to_lowercase();
            let rpr = s.child("w:rPr");
            let outline = s
                .child("w:pPr")
                .and_then(|p| p.child("w:outlineLvl"))
                .and_then(|o| o.attr("w:val"))
                .and_then(|v| v.parse::<u8>().ok());
            map.insert(
                id.to_string(),
                StyleInfo {
                    bold: on(rpr.and_then(|r| r.child("w:b"))) || name == "strong",
                    italic: on(rpr.and_then(|r| r.child("w:i"))) || name == "emphasis",
                    outline,
                    name,
                },
            );
        }
    }
    Ok(map)
}

fn read_numbering(input: &[u8]) -> Result<HashMap<String, Levels>, String> {
    let mut out = HashMap::new();
    let Some(x) = xml::zip_entry(input, "word/numbering.xml")? else {
        return Ok(out);
    };
    let root = xml::parse(&x)?;
    let mut abstracts: HashMap<String, Levels> = HashMap::new();
    for a in root.elements().filter(|e| e.name == "w:abstractNum") {
        let Some(id) = a.attr("w:abstractNumId") else {
            continue;
        };
        let mut levels = HashMap::new();
        for l in a.elements().filter(|e| e.name == "w:lvl") {
            let ilvl = l.attr("w:ilvl").and_then(|v| v.parse().ok()).unwrap_or(0);
            let fmt = l
                .child("w:numFmt")
                .and_then(|f| f.attr("w:val"))
                .unwrap_or("bullet");
            let start = l
                .child("w:start")
                .and_then(|s| s.attr("w:val"))
                .and_then(|v| v.parse().ok())
                .unwrap_or(1);
            levels.insert(ilvl, (!matches!(fmt, "bullet" | "none"), start));
        }
        abstracts.insert(id.to_string(), levels);
    }
    for n in root.elements().filter(|e| e.name == "w:num") {
        let (Some(id), Some(abs)) = (
            n.attr("w:numId"),
            n.child("w:abstractNumId").and_then(|a| a.attr("w:val")),
        ) else {
            continue;
        };
        if let Some(levels) = abstracts.get(abs) {
            out.insert(id.to_string(), levels.clone());
        }
    }
    Ok(out)
}

fn body_items(parent: &Element, ctx: &Ctx, out: &mut Vec<Item>) {
    for el in parent.elements() {
        match el.name.as_str() {
            "w:p" => paragraph(el, ctx, out),
            "w:tbl" => out.push(Item::Block(table(el, ctx))),
            "w:sdt" => {
                if let Some(content) = el.child("w:sdtContent") {
                    body_items(content, ctx, out);
                }
            }
            _ => {}
        }
    }
}

fn paragraph(p: &Element, ctx: &Ctx, out: &mut Vec<Item>) {
    let ppr = p.child("w:pPr");
    let style_id = ppr
        .and_then(|x| x.child("w:pStyle"))
        .and_then(|s| s.attr("w:val"))
        .unwrap_or("");
    let style = ctx.styles.get(style_id);
    let name = style.map(|s| s.name.as_str()).unwrap_or("");

    let mut inl = Vec::new();
    runs(p, ctx, Style::default(), None, &mut inl);

    let code_style =
        name.contains("preformatted") || name.contains("code") || name == "source text";
    if code_style {
        let text: String = inl.iter().map(|s| s.text.as_str()).collect();
        // Consecutive code paragraphs merge into one block.
        if let Some(Item::Block(Block::Code(prev))) = out.last_mut() {
            prev.push('\n');
            prev.push_str(&text);
        } else {
            out.push(Item::Block(Block::Code(text)));
        }
        return;
    }

    let inl = normalize_inlines(inl);

    let heading = if let Some(n) = name.strip_prefix("heading ") {
        n.trim().parse::<u8>().ok()
    } else if name == "title" {
        Some(1)
    } else if name == "subtitle" {
        Some(2)
    } else {
        ppr.and_then(|x| x.child("w:outlineLvl"))
            .and_then(|o| o.attr("w:val"))
            .and_then(|v| v.parse::<u8>().ok())
            .or(style.and_then(|s| s.outline))
            .filter(|l| *l < 9)
            .map(|l| l + 1)
    };

    if let Some(level) = heading {
        if !inl.is_empty() {
            out.push(Item::Block(Block::Heading(level.min(6), inl)));
        }
        return;
    }

    if let Some(num) = ppr.and_then(|x| x.child("w:numPr")) {
        let num_id = num
            .child("w:numId")
            .and_then(|n| n.attr("w:val"))
            .unwrap_or("0");
        let level: u8 = num
            .child("w:ilvl")
            .and_then(|n| n.attr("w:val"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        // numId 0 means "numbering removed".
        if num_id != "0" {
            let (ordered, start) = ctx
                .numbering
                .get(num_id)
                .and_then(|l| l.get(&level))
                .copied()
                .unwrap_or((false, 1));
            out.push(Item::ListPara {
                level,
                ordered,
                start,
                inlines: inl,
            });
            return;
        }
    }

    if name == "quote" || name == "intense quote" || name == "quotations" {
        if !inl.is_empty() {
            out.push(Item::Block(Block::Quote(vec![Block::Para(inl)])));
        }
        return;
    }

    if !inl.is_empty() {
        out.push(Item::Block(Block::Para(inl)));
    } else if ppr
        .and_then(|x| x.child("w:pBdr"))
        .and_then(|b| b.child("w:bottom"))
        .is_some()
    {
        // An empty paragraph with only a bottom border is a horizontal rule.
        out.push(Item::Block(Block::Rule));
    }
}

const MONO_FONTS: &[&str] = &[
    "courier",
    "consolas",
    "menlo",
    "monaco",
    "mono",
    "source code",
    "lucida console",
];

fn runs(parent: &Element, ctx: &Ctx, style: Style, link: Option<&str>, out: &mut Inlines) {
    for el in parent.elements() {
        match el.name.as_str() {
            "w:r" => {
                let mut st = style;
                if let Some(rpr) = el.child("w:rPr") {
                    if let Some(s) = rpr
                        .child("w:rStyle")
                        .and_then(|s| s.attr("w:val"))
                        .and_then(|id| ctx.styles.get(id))
                    {
                        st.bold |= s.bold;
                        st.italic |= s.italic;
                        st.code |= s.name.contains("code") || s.name == "source text";
                    }
                    if rpr.child("w:b").is_some() {
                        st.bold = on(rpr.child("w:b"));
                    }
                    if rpr.child("w:i").is_some() {
                        st.italic = on(rpr.child("w:i"));
                    }
                    st.strike |= on(rpr.child("w:strike")) || on(rpr.child("w:dstrike"));
                    if let Some(font) = rpr.child("w:rFonts").and_then(|f| f.attr("w:ascii")) {
                        let f = font.to_lowercase();
                        st.code |= MONO_FONTS.iter().any(|m| f.contains(m));
                    }
                }
                for c in el.elements() {
                    match c.name.as_str() {
                        "w:t" => push_text(out, &c.text(), st, link),
                        "w:tab" | "w:ptab" => push_text(out, "\t", st, link),
                        "w:br" | "w:cr" => {
                            if c.attr("w:type") != Some("page") {
                                out.push(Span::line_break());
                            }
                        }
                        "w:noBreakHyphen" => push_text(out, "-", st, link),
                        _ => {}
                    }
                }
            }
            "w:hyperlink" => {
                let target = el
                    .attr("r:id")
                    .and_then(|id| ctx.rels.get(id))
                    .map(String::as_str);
                runs(el, ctx, style, target.or(link), out);
            }
            // Tracked insertions, smart tags, content controls and simple
            // fields all wrap ordinary runs.
            "w:ins" | "w:smartTag" | "w:fldSimple" | "w:customXml" => {
                runs(el, ctx, style, link, out)
            }
            "w:sdt" => {
                if let Some(c) = el.child("w:sdtContent") {
                    runs(c, ctx, style, link, out);
                }
            }
            _ => {}
        }
    }
}

fn table(tbl: &Element, ctx: &Ctx) -> Block {
    let mut rows = Vec::new();
    for tr in tbl.elements().filter(|e| e.name == "w:tr") {
        let mut row = Vec::new();
        for tc in tr.elements().filter(|e| e.name == "w:tc") {
            let mut cell: Inlines = Vec::new();
            for p in tc.elements().filter(|e| e.name == "w:p") {
                let mut inl = Vec::new();
                runs(p, ctx, Style::default(), None, &mut inl);
                let inl = normalize_inlines(inl);
                if inl.is_empty() {
                    continue;
                }
                if !cell.is_empty() {
                    cell.push(Span::line_break());
                }
                cell.extend(inl);
            }
            row.push(cell);
        }
        rows.push(row);
    }
    Block::Table(rows)
}

/// Turn flat numbered paragraphs into nested lists.
fn group_lists(items: Vec<Item>) -> Vec<Block> {
    let mut out = Vec::new();
    let mut run: Vec<(u8, bool, u32, Inlines)> = Vec::new();
    for item in items {
        match item {
            Item::ListPara {
                level,
                ordered,
                start,
                inlines,
            } => run.push((level, ordered, start, inlines)),
            Item::Block(b) => {
                if !run.is_empty() {
                    out.extend(build_list(&run, 0));
                    run.clear();
                }
                out.push(b);
            }
        }
    }
    if !run.is_empty() {
        out.extend(build_list(&run, 0));
    }
    out
}

pub(super) fn build_list(items: &[(u8, bool, u32, Inlines)], base: u8) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < items.len() {
        let (_, ordered, start, _) = items[i];
        let mut list_items: Vec<Vec<Block>> = Vec::new();
        while i < items.len() {
            let (level, ord, _, inl) = &items[i];
            if *level > base && !list_items.is_empty() {
                // Deeper items, of either list type, nest under the previous item.
                let mut j = i;
                while j < items.len() && items[j].0 > base {
                    j += 1;
                }
                let nested = build_list(&items[i..j], base + 1);
                list_items.last_mut().expect("non-empty").extend(nested);
                i = j;
                continue;
            }
            // A different list type at the same level starts a new list.
            if *ord != ordered && !list_items.is_empty() {
                break;
            }
            list_items.push(vec![Block::Para(inl.clone())]);
            i += 1;
        }
        blocks.push(Block::List {
            ordered,
            start,
            items: list_items,
        });
    }
    blocks
}

// ── Writer ──────────────────────────────────────────────

use docx_rs::{
    AbstractNumbering, AlignmentType, BorderType, BreakType, Docx, Hyperlink, HyperlinkType,
    IndentLevel, Level, LevelJc, LevelOverride, LevelText, LineSpacing, NumberFormat, Numbering,
    NumberingId, Paragraph, ParagraphBorder, ParagraphBorderPosition, ParagraphBorders, Run,
    RunFonts, Shading, SpecialIndentType, Start, Style as DStyle, StyleType, Table, TableCell,
    TableRow, WidthType,
};

// docx-rs ships its own default numbering with id 1; stay clear of it.
const BULLET_NUM: usize = 100;
const ORDERED_ABSTRACT: usize = 101;
const MONO: &str = "Consolas";

struct Writer {
    docx: Docx,
    next_num: usize,
}

pub fn write(doc: &Doc) -> Result<Vec<u8>, String> {
    let bullets = ["\u{2022}", "\u{25e6}", "\u{25aa}"];
    let mut bullet = AbstractNumbering::new(BULLET_NUM);
    let mut ordered = AbstractNumbering::new(ORDERED_ABSTRACT);
    for l in 0..9usize {
        let left = 720 * (l as i32 + 1);
        bullet = bullet.add_level(
            Level::new(
                l,
                Start::new(1),
                NumberFormat::new("bullet"),
                LevelText::new(bullets[l % 3]),
                LevelJc::new("left"),
            )
            .indent(
                Some(left),
                Some(SpecialIndentType::Hanging(360)),
                None,
                None,
            ),
        );
        let fmt = ["decimal", "lowerLetter", "lowerRoman"][l % 3];
        ordered = ordered.add_level(
            Level::new(
                l,
                Start::new(1),
                NumberFormat::new(fmt),
                LevelText::new(format!("%{}.", l + 1)),
                LevelJc::new("left"),
            )
            .indent(
                Some(left),
                Some(SpecialIndentType::Hanging(360)),
                None,
                None,
            ),
        );
    }

    let heading_sizes = [40, 32, 26, 24, 22, 22];
    let mut docx = Docx::new()
        .default_fonts(
            RunFonts::new()
                .ascii("Calibri")
                .hi_ansi("Calibri")
                .cs("Calibri")
                .east_asia("Calibri"),
        )
        .default_size(22)
        .add_style(
            DStyle::new("Hyperlink", StyleType::Character)
                .name("Hyperlink")
                .color("0563C1")
                .underline("single"),
        )
        .add_style(
            DStyle::new("Quote", StyleType::Paragraph)
                .name("Quote")
                .italic()
                .color("57534E")
                .indent(Some(720), None, None, None),
        )
        .add_style(
            DStyle::new("SourceCode", StyleType::Paragraph)
                .name("Source Code")
                .fonts(RunFonts::new().ascii(MONO).hi_ansi(MONO).cs(MONO))
                .size(19)
                .line_spacing(LineSpacing::new().before(0).after(0)),
        )
        .add_abstract_numbering(bullet)
        .add_abstract_numbering(ordered)
        .add_numbering(Numbering::new(BULLET_NUM, BULLET_NUM));
    for (i, size) in heading_sizes.iter().enumerate() {
        let n = i + 1;
        let mut s = DStyle::new(format!("Heading{n}"), StyleType::Paragraph)
            .name(format!("heading {n}"))
            .based_on("Normal")
            .next("Normal")
            .bold()
            .size(*size)
            .outline_lvl(i)
            .line_spacing(
                LineSpacing::new()
                    .before(if n == 1 { 360 } else { 240 })
                    .after(120),
            );
        if n == 6 {
            s = s.italic();
        }
        docx = docx.add_style(s);
    }

    let mut w = Writer {
        docx,
        next_num: ORDERED_ABSTRACT + 1,
    };
    for b in &doc.blocks {
        w.block(b, 0, None);
    }
    let mut buf = Cursor::new(Vec::new());
    w.docx
        .build()
        .pack(&mut buf)
        .map_err(|e| format!("Failed to write DOCX: {e}"))?;
    Ok(buf.into_inner())
}

impl Writer {
    fn push(&mut self, p: Paragraph) {
        let docx = std::mem::take(&mut self.docx);
        self.docx = docx.add_paragraph(p);
    }

    /// `list` is the numbering to apply to the first paragraph of a list item.
    fn block(&mut self, b: &Block, depth: usize, list: Option<(usize, usize)>) {
        match b {
            Block::Heading(level, inl) => {
                let p = Paragraph::new().style(&format!("Heading{}", (*level).clamp(1, 6)));
                self.push(add_inlines(p, inl));
            }
            Block::Para(inl) => {
                let mut p = Paragraph::new().line_spacing(LineSpacing::new().after(120));
                if let Some((num, level)) = list {
                    p = p
                        .numbering(NumberingId::new(num), IndentLevel::new(level))
                        .line_spacing(LineSpacing::new().after(40));
                } else if depth > 0 {
                    p = p.indent(Some(720 * depth as i32), None, None, None);
                }
                self.push(add_inlines(p, inl));
            }
            Block::List {
                ordered,
                start,
                items,
            } => {
                let num = if *ordered {
                    let id = self.next_num;
                    self.next_num += 1;
                    let mut n = Numbering::new(id, ORDERED_ABSTRACT);
                    for l in 0..9 {
                        n = n.add_override(LevelOverride::new(l).start(if l == depth {
                            *start as usize
                        } else {
                            1
                        }));
                    }
                    let docx = std::mem::take(&mut self.docx);
                    self.docx = docx.add_numbering(n);
                    id
                } else {
                    BULLET_NUM
                };
                for item in items {
                    for (i, child) in item.iter().enumerate() {
                        let numbering = (i == 0).then_some((num, depth.min(8)));
                        match child {
                            Block::List { .. } => self.block(child, depth + 1, None),
                            _ => self.block(child, depth + 1, numbering),
                        }
                    }
                }
            }
            Block::Quote(inner) => {
                for child in inner {
                    match child {
                        Block::Para(inl) => {
                            self.push(add_inlines(Paragraph::new().style("Quote"), inl))
                        }
                        other => self.block(other, depth + 1, None),
                    }
                }
            }
            Block::Code(code) => {
                let mut run = Run::new().shading(Shading::new().fill("F2F2F2"));
                for (i, line) in code.lines().enumerate() {
                    if i > 0 {
                        run = run.add_break(BreakType::TextWrapping);
                    }
                    run = run.add_text(line);
                }
                self.push(
                    Paragraph::new()
                        .style("SourceCode")
                        .add_run(run)
                        .line_spacing(LineSpacing::new().after(160)),
                );
            }
            Block::Table(rows) => {
                let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
                let rows: Vec<TableRow> = rows
                    .iter()
                    .enumerate()
                    .map(|(r, row)| {
                        let cells = (0..cols)
                            .map(|c| {
                                let empty = Vec::new();
                                let inl = row.get(c).unwrap_or(&empty);
                                let mut p = add_inlines(Paragraph::new(), inl);
                                if r == 0 {
                                    p = p.bold();
                                }
                                let mut cell = TableCell::new().add_paragraph(p);
                                if r == 0 {
                                    cell = cell.shading(Shading::new().fill("F2F2F2"));
                                }
                                cell
                            })
                            .collect();
                        TableRow::new(cells)
                    })
                    .collect();
                let docx = std::mem::take(&mut self.docx);
                self.docx = docx
                    .add_table(Table::new(rows).width(5000, WidthType::Pct))
                    .add_paragraph(Paragraph::new());
            }
            Block::Rule => {
                let border = ParagraphBorder::new(ParagraphBorderPosition::Bottom)
                    .val(BorderType::Single)
                    .size(6)
                    .color("BFBFBF");
                self.push(
                    Paragraph::new()
                        .set_borders(ParagraphBorders::with_empty().set(border))
                        .align(AlignmentType::Left),
                );
            }
        }
    }
}

fn styled_run(span: &Span) -> Run {
    let mut run = Run::new();
    for (i, part) in span.text.split('\t').enumerate() {
        if i > 0 {
            run = run.add_tab();
        }
        if !part.is_empty() {
            run = run.add_text(part);
        }
    }
    let st = span.style;
    if st.bold {
        run = run.bold();
    }
    if st.italic {
        run = run.italic();
    }
    if st.strike {
        run = run.strike();
    }
    if st.code {
        run = run
            .fonts(RunFonts::new().ascii(MONO).hi_ansi(MONO).cs(MONO))
            .size(20)
            .shading(Shading::new().fill("F2F2F2"));
    }
    run
}

fn add_inlines(mut p: Paragraph, inl: &[Span]) -> Paragraph {
    let mut i = 0;
    while i < inl.len() {
        let span = &inl[i];
        if span.is_break() {
            p = p.add_run(Run::new().add_break(BreakType::TextWrapping));
            i += 1;
        } else if let Some(url) = &span.link {
            let mut link = Hyperlink::new(url, HyperlinkType::External);
            while i < inl.len() && inl[i].link.as_ref() == Some(url) {
                link = link.add_run(styled_run(&inl[i]).style("Hyperlink"));
                i += 1;
            }
            p = p.add_hyperlink(link);
        } else {
            p = p.add_run(styled_run(span));
            i += 1;
        }
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_lists_group_by_level() {
        let items = vec![
            (0, false, 1, vec![Span::plain("a")]),
            (1, true, 1, vec![Span::plain("a.1")]),
            (0, false, 1, vec![Span::plain("b")]),
        ];
        let blocks = build_list(&items, 0);
        assert_eq!(blocks.len(), 1);
        let Block::List { items, ordered, .. } = &blocks[0] else {
            panic!()
        };
        assert!(!ordered);
        assert_eq!(items.len(), 2);
        assert!(matches!(items[0][1], Block::List { ordered: true, .. }));
    }

    #[test]
    fn write_then_read_roundtrip() {
        let doc = super::super::markdown::read(
            "# Title\n\nSome **bold** and *italic* with a [link](https://example.com).\n\n- one\n- two\n  1. nested\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n",
        );
        let bytes = write(&doc).unwrap();
        let back = read(&bytes).unwrap();
        assert_eq!(back, doc);
    }
}
