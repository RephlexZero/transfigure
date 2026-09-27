//! PDF writing: a small layout engine for documents, and image pages.
//!
//! Text uses the PDF standard fonts (Helvetica, Courier) with WinAnsi
//! encoding, so nothing needs embedding. Line breaking uses real glyph
//! widths, which is what keeps wrapped text inside the margins.

use std::io::Write;

use unicode_normalization::UnicodeNormalization;

use crate::doc::{Block, Doc, Span, Style};

// ── Fonts ───────────────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Font {
    Regular,
    Bold,
    Italic,
    BoldItalic,
    Mono,
    MonoBold,
}

const FONTS: [(Font, &str); 6] = [
    (Font::Regular, "Helvetica"),
    (Font::Bold, "Helvetica-Bold"),
    (Font::Italic, "Helvetica-Oblique"),
    (Font::BoldItalic, "Helvetica-BoldOblique"),
    (Font::Mono, "Courier"),
    (Font::MonoBold, "Courier-Bold"),
];

impl Font {
    fn for_style(st: Style) -> Font {
        match (st.code, st.bold, st.italic) {
            (true, true, _) => Font::MonoBold,
            (true, false, _) => Font::Mono,
            (false, true, true) => Font::BoldItalic,
            (false, true, false) => Font::Bold,
            (false, false, true) => Font::Italic,
            (false, false, false) => Font::Regular,
        }
    }

    fn resource(self) -> usize {
        FONTS.iter().position(|(f, _)| *f == self).expect("listed") + 1
    }

    /// Advance width of a WinAnsi byte in 1/1000 em.
    fn width(self, b: u8) -> f32 {
        if b < 32 {
            return 0.0;
        }
        let table = match self {
            Font::Mono | Font::MonoBold => return 600.0,
            Font::Bold | Font::BoldItalic => &metrics::HELVETICA_BOLD,
            Font::Regular | Font::Italic => &metrics::HELVETICA,
        };
        let w = table[(b - 32) as usize];
        if w == 0 { 556.0 } else { w as f32 }
    }

    fn measure(self, bytes: &[u8], size: f32) -> f32 {
        bytes.iter().map(|b| self.width(*b)).sum::<f32>() * size / 1000.0
    }
}

/// Encode text as WinAnsi (Windows-1252) bytes. Characters outside it fall
/// back to their base letter (č → c) or a close ASCII stand-in.
fn winansi(text: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    for c in text.chars() {
        if let Some(b) = winansi_char(c) {
            out.push(b);
            continue;
        }
        let fallback = match c {
            '\t' => "    ",
            '\u{2212}' | '\u{2010}' | '\u{2011}' => "-",
            '\u{2192}' => "->",
            '\u{2190}' => "<-",
            '\u{2194}' => "<->",
            '\u{21d2}' => "=>",
            '\u{2264}' => "<=",
            '\u{2265}' => ">=",
            '\u{2260}' => "!=",
            '\u{2248}' => "~",
            '\u{2713}' | '\u{2714}' => "v",
            '\u{2717}' | '\u{2718}' => "x",
            '\u{25e6}' | '\u{25cb}' => "o",
            '\u{25aa}' | '\u{25a0}' => "-",
            '\u{2032}' => "'",
            '\u{2033}' => "\"",
            '\u{2009}' | '\u{200a}' | '\u{2002}' | '\u{2003}' | '\u{202f}' => " ",
            '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{feff}' => "",
            _ => {
                // Strip diacritics the encoding lacks.
                let base: String = c.nfd().filter(|d| winansi_char(*d).is_some()).collect();
                if !base.is_empty() {
                    out.extend(base.chars().filter_map(winansi_char));
                } else {
                    out.push(b'?');
                }
                continue;
            }
        };
        out.extend_from_slice(fallback.as_bytes());
    }
    out
}

fn winansi_char(c: char) -> Option<u8> {
    let u = c as u32;
    if (0x20..=0x7e).contains(&u) || (0xa0..=0xff).contains(&u) {
        return Some(u as u8);
    }
    Some(match c {
        '€' => 0x80,
        '‚' => 0x82,
        'ƒ' => 0x83,
        '„' => 0x84,
        '…' => 0x85,
        '†' => 0x86,
        '‡' => 0x87,
        'ˆ' => 0x88,
        '‰' => 0x89,
        'Š' => 0x8a,
        '‹' => 0x8b,
        'Œ' => 0x8c,
        'Ž' => 0x8e,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '•' => 0x95,
        '–' => 0x96,
        '—' => 0x97,
        '˜' => 0x98,
        '™' => 0x99,
        'š' => 0x9a,
        '›' => 0x9b,
        'œ' => 0x9c,
        'ž' => 0x9e,
        'Ÿ' => 0x9f,
        _ => return None,
    })
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2 + 2);
    s.push('<');
    for b in bytes {
        s.push_str(&format!("{b:02X}"));
    }
    s.push('>');
    s
}

/// A PDF text string (for metadata), UTF-16BE so any title survives.
fn text_string(s: &str) -> String {
    let mut bytes = vec![0xfe, 0xff];
    for u in s.encode_utf16() {
        bytes.extend_from_slice(&u.to_be_bytes());
    }
    hex(&bytes)
}

// ── Object assembly ─────────────────────────────────────

struct Builder {
    objects: Vec<Vec<u8>>,
}

impl Builder {
    fn new() -> Self {
        Builder {
            objects: Vec::new(),
        }
    }

    /// Reserve an object number to fill in later.
    fn reserve(&mut self) -> usize {
        self.objects.push(Vec::new());
        self.objects.len()
    }

    fn set(&mut self, id: usize, body: Vec<u8>) {
        self.objects[id - 1] = body;
    }

    fn add(&mut self, body: impl Into<Vec<u8>>) -> usize {
        self.objects.push(body.into());
        self.objects.len()
    }

    fn stream(&mut self, dict: &str, data: &[u8], compress: bool) -> usize {
        let (data, filter) = if compress {
            let mut enc =
                flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
            enc.write_all(data).expect("in-memory write");
            (
                enc.finish().expect("in-memory write"),
                " /Filter /FlateDecode",
            )
        } else {
            (data.to_vec(), "")
        };
        let mut body =
            format!("<< {dict}{filter} /Length {} >>\nstream\n", data.len()).into_bytes();
        body.extend_from_slice(&data);
        body.extend_from_slice(b"\nendstream");
        self.add(body)
    }

    fn finish(self, catalog: usize, info: usize) -> Vec<u8> {
        let mut pdf = b"%PDF-1.4\n%\xe2\xe3\xcf\xd3\n".to_vec();
        let mut offsets = Vec::with_capacity(self.objects.len());
        for (i, body) in self.objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
            pdf.extend_from_slice(body);
            pdf.extend_from_slice(b"\nendobj\n");
        }
        let xref = pdf.len();
        pdf.extend_from_slice(format!("xref\n0 {}\n", self.objects.len() + 1).as_bytes());
        pdf.extend_from_slice(b"0000000000 65535 f \n");
        for off in offsets {
            pdf.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
        }
        pdf.extend_from_slice(
            format!(
                "trailer\n<< /Size {} /Root {catalog} 0 R /Info {info} 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                self.objects.len() + 1
            )
            .as_bytes(),
        );
        pdf
    }
}

struct Page {
    ops: String,
    links: Vec<([f32; 4], String)>,
    images: Vec<usize>,
}

impl Page {
    fn new() -> Self {
        Page {
            ops: String::new(),
            links: Vec::new(),
            images: Vec::new(),
        }
    }
}

fn assemble(pages: Vec<Page>, width: f32, height: f32, title: Option<&str>) -> Vec<u8> {
    let mut b = Builder::new();
    let catalog = b.reserve();
    let pages_id = b.reserve();
    let font_ids: Vec<usize> = FONTS
        .iter()
        .map(|(_, name)| {
            b.add(format!(
                "<< /Type /Font /Subtype /Type1 /BaseFont /{name} /Encoding /WinAnsiEncoding >>"
            ))
        })
        .collect();
    let fonts: String = font_ids
        .iter()
        .enumerate()
        .map(|(i, id)| format!("/F{} {id} 0 R ", i + 1))
        .collect();

    let mut kids = Vec::new();
    for page in pages {
        let content = b.stream("", page.ops.as_bytes(), true);
        let annots: String = page
            .links
            .iter()
            .map(|(r, url)| {
                format!(
                    "<< /Type /Annot /Subtype /Link /Rect [{:.2} {:.2} {:.2} {:.2}] /Border [0 0 0] /A << /S /URI /URI {} >> >> ",
                    r[0],
                    r[1],
                    r[2],
                    r[3],
                    hex(url.as_bytes())
                )
            })
            .collect();
        let xobjects: String = page
            .images
            .iter()
            .enumerate()
            .map(|(i, id)| format!("/Im{} {id} 0 R ", i + 1))
            .collect();
        let page_id = b.add(format!(
            "<< /Type /Page /Parent {pages_id} 0 R /MediaBox [0 0 {width:.2} {height:.2}] /Contents {content} 0 R \
             /Resources << /Font << {fonts}>> /XObject << {xobjects}>> >> /Annots [{annots}] >>"
        ));
        kids.push(page_id);
    }
    let kids_str: String = kids.iter().map(|k| format!("{k} 0 R ")).collect();
    b.set(
        pages_id,
        format!(
            "<< /Type /Pages /Kids [{kids_str}] /Count {} >>",
            kids.len()
        )
        .into_bytes(),
    );
    b.set(
        catalog,
        format!("<< /Type /Catalog /Pages {pages_id} 0 R >>").into_bytes(),
    );
    let mut info = format!("<< /Producer {}", text_string("Transfigure"));
    if let Some(t) = title {
        info.push_str(&format!(" /Title {}", text_string(t)));
    }
    info.push_str(" >>");
    let info = b.add(info);
    b.finish(catalog, info)
}

// ── Document layout ─────────────────────────────────────

const PAGE_W: f32 = 595.28;
const PAGE_H: f32 = 841.89;
const MARGIN_X: f32 = 64.0;
const MARGIN_TOP: f32 = 72.0;
const MARGIN_BOTTOM: f32 = 72.0;
const CONTENT_W: f32 = PAGE_W - 2.0 * MARGIN_X;

const BODY_SIZE: f32 = 10.5;
const BODY_LEADING: f32 = 15.0;
const INK: &str = "0.110 0.098 0.090";
const MUTED: &str = "0.340 0.325 0.306";
const LINK: &str = "0.020 0.360 0.760";
const CODE_BG: &str = "0.957 0.953 0.945";
const RULE: &str = "0.820 0.808 0.792";

#[derive(Clone)]
struct Frag {
    bytes: Vec<u8>,
    font: Font,
    size: f32,
    x: f32,
    width: f32,
    style: Style,
    link: Option<String>,
}

#[derive(Clone, Default)]
struct Line {
    frags: Vec<Frag>,
    width: f32,
}

/// Break inline spans into lines no wider than `width`.
fn wrap(inl: &[Span], width: f32, size: f32, base: Style) -> Vec<Line> {
    enum Tok {
        Word(Vec<u8>, Style, Option<String>),
        Space(Style, Option<String>),
        Break,
    }
    let mut toks = Vec::new();
    for span in inl {
        if span.is_break() {
            toks.push(Tok::Break);
            continue;
        }
        let st = Style {
            bold: span.style.bold || base.bold,
            italic: span.style.italic || base.italic,
            code: span.style.code || base.code,
            strike: span.style.strike,
        };
        let mut word = String::new();
        for c in span.text.chars() {
            if c.is_whitespace() && c != '\u{a0}' {
                if !word.is_empty() {
                    toks.push(Tok::Word(winansi(&word), st, span.link.clone()));
                    word.clear();
                }
                toks.push(Tok::Space(st, span.link.clone()));
            } else {
                word.push(c);
            }
        }
        if !word.is_empty() {
            toks.push(Tok::Word(winansi(&word), st, span.link.clone()));
        }
    }

    let size_for = |st: Style| if st.code { size * 0.92 } else { size };
    let mut lines = vec![Line::default()];
    let mut pending_space: Option<(Style, Option<String>)> = None;

    let push = |line: &mut Line, bytes: Vec<u8>, st: Style, link: Option<String>| {
        let font = Font::for_style(st);
        let sz = size_for(st);
        let w = font.measure(&bytes, sz);
        // Extend the previous fragment when formatting matches.
        if let Some(last) = line.frags.last_mut()
            && last.style == st
            && last.link == link
        {
            last.bytes.extend_from_slice(&bytes);
            last.width += w;
        } else {
            line.frags.push(Frag {
                bytes,
                font,
                size: sz,
                x: line.width,
                width: w,
                style: st,
                link,
            });
        }
        line.width += w;
    };

    for tok in toks {
        match tok {
            Tok::Break => {
                lines.push(Line::default());
                pending_space = None;
            }
            Tok::Space(st, link) => {
                if !lines.last().expect("non-empty").frags.is_empty() {
                    pending_space = Some((st, link));
                }
            }
            Tok::Word(bytes, st, link) => {
                let font = Font::for_style(st);
                let sz = size_for(st);
                let ww = font.measure(&bytes, sz);
                let line = lines.last_mut().expect("non-empty");
                let space_w = pending_space.as_ref().map_or(0.0, |(s, _)| {
                    Font::for_style(*s).measure(b" ", size_for(*s))
                });
                if !line.frags.is_empty() && line.width + space_w + ww > width {
                    lines.push(Line::default());
                    pending_space = None;
                }
                let line = lines.last_mut().expect("non-empty");
                if let Some((s, l)) = pending_space.take() {
                    push(line, b" ".to_vec(), s, l);
                }
                if ww <= width || !line.frags.is_empty() {
                    push(line, bytes, st, link);
                } else {
                    // A single word wider than the line: break it by glyph.
                    let mut chunk = Vec::new();
                    let mut cw = 0.0;
                    for b in bytes {
                        let gw = font.width(b) * sz / 1000.0;
                        if cw + gw > width && !chunk.is_empty() {
                            push(
                                lines.last_mut().expect("non-empty"),
                                std::mem::take(&mut chunk),
                                st,
                                link.clone(),
                            );
                            lines.push(Line::default());
                            cw = 0.0;
                        }
                        chunk.push(b);
                        cw += gw;
                    }
                    push(lines.last_mut().expect("non-empty"), chunk, st, link);
                }
            }
        }
    }
    if lines.len() > 1 && lines.last().is_some_and(|l| l.frags.is_empty()) {
        lines.pop();
    }
    lines
}

struct Layout {
    pages: Vec<Page>,
    y: f32,
    /// Vertical space requested before the next block, collapsed at page tops.
    gap: f32,
}

impl Layout {
    fn new() -> Self {
        Layout {
            pages: vec![Page::new()],
            y: PAGE_H - MARGIN_TOP,
            gap: 0.0,
        }
    }

    fn page(&mut self) -> &mut Page {
        self.pages.last_mut().expect("at least one page")
    }

    fn new_page(&mut self) {
        self.pages.push(Page::new());
        self.y = PAGE_H - MARGIN_TOP;
        self.gap = 0.0;
    }

    fn at_top(&self) -> bool {
        self.y >= PAGE_H - MARGIN_TOP - 0.01
    }

    /// Make room for `h` points (plus any pending gap), breaking the page if needed.
    fn reserve(&mut self, h: f32) {
        let gap = if self.at_top() { 0.0 } else { self.gap };
        if self.y - gap - h < MARGIN_BOTTOM && !self.at_top() {
            self.new_page();
        } else {
            self.y -= gap;
        }
        self.gap = 0.0;
    }

    fn space(&mut self, pts: f32) {
        self.gap = self.gap.max(pts);
    }

    fn draw_line(&mut self, line: &Line, x: f32, baseline: f32, color: &str) {
        let page = self.pages.last_mut().expect("page");
        for f in &line.frags {
            let fx = x + f.x;
            if f.style.code && f.bytes.iter().any(|b| *b != b' ') {
                page.ops.push_str(&format!(
                    "{CODE_BG} rg {:.2} {:.2} {:.2} {:.2} re f\n",
                    fx - 1.0,
                    baseline - f.size * 0.25,
                    f.width + 2.0,
                    f.size * 1.15
                ));
            }
            let fill = if f.link.is_some() { LINK } else { color };
            page.ops.push_str(&format!(
                "BT /F{} {:.2} Tf {fill} rg {fx:.2} {baseline:.2} Td {} Tj ET\n",
                f.font.resource(),
                f.size,
                hex(&f.bytes)
            ));
            if f.link.is_some() || f.style.strike {
                let ly = if f.style.strike {
                    baseline + f.size * 0.3
                } else {
                    baseline - f.size * 0.12
                };
                let stroke = if f.link.is_some() { LINK } else { color };
                page.ops.push_str(&format!(
                    "{stroke} RG 0.6 w {fx:.2} {ly:.2} m {:.2} {ly:.2} l S\n",
                    fx + f.width
                ));
            }
            if let Some(url) = &f.link {
                page.links.push((
                    [
                        fx,
                        baseline - f.size * 0.3,
                        fx + f.width,
                        baseline + f.size * 0.9,
                    ],
                    url.clone(),
                ));
            }
        }
    }

    /// Lay out a paragraph. `marker` is drawn left of the first line (list bullets).
    #[allow(clippy::too_many_arguments)]
    fn paragraph(
        &mut self,
        inl: &[Span],
        x: f32,
        width: f32,
        size: f32,
        leading: f32,
        base: Style,
        color: &str,
        marker: Option<&str>,
        bar: Option<f32>,
    ) {
        let lines = wrap(inl, width, size, base);
        for (i, line) in lines.iter().enumerate() {
            self.reserve(leading);
            let baseline = self.y - leading * 0.78;
            if i == 0
                && let Some(m) = marker
            {
                let bytes = winansi(m);
                let mw = Font::Regular.measure(&bytes, size);
                self.page().ops.push_str(&format!(
                    "BT /F1 {size:.2} Tf {MUTED} rg {:.2} {baseline:.2} Td {} Tj ET\n",
                    x - mw - 6.0,
                    hex(&bytes)
                ));
            }
            if let Some(bx) = bar {
                let y = self.y;
                self.page().ops.push_str(&format!(
                    "{RULE} rg {bx:.2} {:.2} 2 {leading:.2} re f\n",
                    y - leading
                ));
            }
            self.draw_line(line, x, baseline, color);
            self.y -= leading;
        }
    }

    fn blocks(&mut self, blocks: &[Block], x: f32, width: f32, quote_bar: Option<f32>) {
        for b in blocks {
            self.block(b, x, width, quote_bar);
        }
    }

    fn block(&mut self, b: &Block, x: f32, width: f32, bar: Option<f32>) {
        match b {
            Block::Heading(level, inl) => {
                let (size, before) = match level {
                    1 => (21.0, 20.0),
                    2 => (16.0, 18.0),
                    3 => (13.0, 14.0),
                    4 => (11.5, 12.0),
                    _ => (10.5, 10.0),
                };
                let leading = size * 1.28;
                self.space(before);
                // Keep a heading with at least two lines of what follows.
                self.reserve(leading + BODY_LEADING * 2.0);
                let st = Style {
                    bold: true,
                    italic: *level >= 6,
                    ..Style::default()
                };
                self.paragraph(inl, x, width, size, leading, st, INK, None, bar);
                self.space(if *level <= 2 { 8.0 } else { 5.0 });
            }
            Block::Para(inl) => {
                self.paragraph(
                    inl,
                    x,
                    width,
                    BODY_SIZE,
                    BODY_LEADING,
                    Style::default(),
                    if bar.is_some() { MUTED } else { INK },
                    None,
                    bar,
                );
                self.space(8.0);
            }
            Block::List {
                ordered,
                start,
                items,
            } => {
                let indent = 18.0;
                for (i, item) in items.iter().enumerate() {
                    let marker = if *ordered {
                        format!("{}.", *start as usize + i)
                    } else {
                        "\u{2022}".to_string()
                    };
                    for (j, child) in item.iter().enumerate() {
                        match (j, child) {
                            (0, Block::Para(inl)) => {
                                self.paragraph(
                                    inl,
                                    x + indent,
                                    width - indent,
                                    BODY_SIZE,
                                    BODY_LEADING,
                                    Style::default(),
                                    INK,
                                    Some(&marker),
                                    bar,
                                );
                                self.space(3.0);
                            }
                            _ => self.block(child, x + indent, width - indent, bar),
                        }
                    }
                }
                self.space(8.0);
            }
            Block::Quote(inner) => {
                self.blocks(inner, x + 16.0, width - 16.0, Some(x + 2.0));
                self.space(8.0);
            }
            Block::Code(code) => {
                let size = 9.0;
                let leading = 12.6;
                let pad = 8.0;
                let cols = ((width - 2.0 * pad) / (size * 0.6)).floor().max(10.0) as usize;
                let mut lines = Vec::new();
                for raw in code.replace('\t', "    ").lines() {
                    let chars: Vec<char> = raw.chars().collect();
                    if chars.is_empty() {
                        lines.push(String::new());
                    }
                    for chunk in chars.chunks(cols) {
                        lines.push(chunk.iter().collect());
                    }
                }
                self.space(4.0);
                for (i, text) in lines.iter().enumerate() {
                    let top_pad = if i == 0 { pad } else { 0.0 };
                    let bottom_pad = if i + 1 == lines.len() { pad } else { 0.0 };
                    let h = leading + top_pad + bottom_pad;
                    self.reserve(h);
                    let y = self.y;
                    self.page().ops.push_str(&format!(
                        "{CODE_BG} rg {x:.2} {:.2} {width:.2} {h:.2} re f\n",
                        y - h
                    ));
                    let baseline = y - top_pad - leading * 0.78;
                    self.page().ops.push_str(&format!(
                        "BT /F5 {size:.2} Tf {INK} rg {:.2} {baseline:.2} Td {} Tj ET\n",
                        x + pad,
                        hex(&winansi(text))
                    ));
                    self.y -= h;
                }
                self.space(10.0);
            }
            Block::Table(rows) => self.table(rows, x, width),
            Block::Rule => {
                self.space(8.0);
                self.reserve(8.0);
                let y = self.y - 4.0;
                self.page().ops.push_str(&format!(
                    "{RULE} RG 0.75 w {x:.2} {y:.2} m {:.2} {y:.2} l S\n",
                    x + width
                ));
                self.y -= 8.0;
                self.space(8.0);
            }
        }
    }

    fn table(&mut self, rows: &[Vec<Vec<Span>>], x: f32, width: f32) {
        let cols = rows.iter().map(Vec::len).max().unwrap_or(0);
        if cols == 0 {
            return;
        }
        let size = 9.5;
        let leading = 13.0;
        let pad = 5.0;

        // Natural (unwrapped) and minimum (longest word) widths per column.
        let mut natural = vec![0.0f32; cols];
        let mut minimum = vec![0.0f32; cols];
        for (r, row) in rows.iter().enumerate() {
            let base = Style {
                bold: r == 0,
                ..Style::default()
            };
            for (c, cell) in row.iter().enumerate() {
                let line = wrap(cell, f32::MAX, size, base);
                let nat = line.iter().map(|l| l.width).fold(0.0, f32::max);
                natural[c] = natural[c].max(nat);
                for span in cell {
                    let font = Font::for_style(Style {
                        bold: span.style.bold || base.bold,
                        ..span.style
                    });
                    for w in span.text.split_whitespace() {
                        minimum[c] =
                            minimum[c].max(font.measure(&winansi(w), size).min(width / 3.0));
                    }
                }
            }
        }
        let avail = width - 2.0 * pad * cols as f32;
        let total_nat: f32 = natural.iter().sum();
        let widths: Vec<f32> = if total_nat <= avail {
            natural.clone()
        } else {
            let total_min: f32 = minimum.iter().sum();
            let extra = (avail - total_min).max(0.0);
            let flex: f32 = natural
                .iter()
                .zip(&minimum)
                .map(|(n, m)| (n - m).max(0.0))
                .sum::<f32>()
                .max(1.0);
            natural
                .iter()
                .zip(&minimum)
                .map(|(n, m)| m + extra * (n - m).max(0.0) / flex)
                .collect()
        };
        let table_w: f32 = widths.iter().sum::<f32>() + 2.0 * pad * cols as f32;

        let layout_row = |r: usize, row: &Vec<Vec<Span>>| -> (Vec<Vec<Line>>, f32) {
            let base = Style {
                bold: r == 0,
                ..Style::default()
            };
            let cells: Vec<Vec<Line>> = (0..cols)
                .map(|c| {
                    let empty = Vec::new();
                    wrap(row.get(c).unwrap_or(&empty), widths[c], size, base)
                })
                .collect();
            let n = cells.iter().map(Vec::len).max().unwrap_or(1).max(1);
            (cells, n as f32 * leading + 2.0 * pad)
        };

        self.space(4.0);
        let header = layout_row(0, &rows[0]);
        for (r, row) in rows.iter().enumerate() {
            let (cells, h) = if r == 0 {
                header.clone()
            } else {
                layout_row(r, row)
            };
            let top_before = self.at_top();
            self.reserve(h);
            // Repeat the header row at the top of a continuation page.
            if r > 0 && self.at_top() && !top_before {
                self.draw_row(&header.0, header.1, x, &widths, pad, leading, table_w, true);
            }
            self.draw_row(&cells, h, x, &widths, pad, leading, table_w, r == 0);
        }
        self.space(10.0);
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_row(
        &mut self,
        cells: &[Vec<Line>],
        h: f32,
        x: f32,
        widths: &[f32],
        pad: f32,
        leading: f32,
        table_w: f32,
        header: bool,
    ) {
        let y = self.y;
        if header {
            self.page().ops.push_str(&format!(
                "{CODE_BG} rg {x:.2} {:.2} {table_w:.2} {h:.2} re f\n",
                y - h
            ));
        }
        let mut cx = x;
        for (c, lines) in cells.iter().enumerate() {
            for (i, line) in lines.iter().enumerate() {
                let baseline = y - pad - leading * i as f32 - leading * 0.78;
                self.draw_line(line, cx + pad, baseline, INK);
            }
            cx += widths[c] + 2.0 * pad;
        }
        let by = y - h;
        self.page().ops.push_str(&format!(
            "{RULE} RG 0.5 w {x:.2} {by:.2} m {:.2} {by:.2} l S\n",
            x + table_w
        ));
        if header {
            self.page().ops.push_str(&format!(
                "{RULE} RG 0.5 w {x:.2} {y:.2} m {:.2} {y:.2} l S\n",
                x + table_w
            ));
        }
        self.y -= h;
    }
}

pub fn render_doc(doc: &Doc) -> Vec<u8> {
    let mut layout = Layout::new();
    layout.blocks(&doc.blocks, MARGIN_X, CONTENT_W, None);
    let total = layout.pages.len();
    if total > 1 {
        for (i, page) in layout.pages.iter_mut().enumerate() {
            let label = winansi(&format!("{} / {total}", i + 1));
            let w = Font::Regular.measure(&label, 8.0);
            page.ops.push_str(&format!(
                "BT /F1 8 Tf {MUTED} rg {:.2} {:.2} Td {} Tj ET\n",
                (PAGE_W - w) / 2.0,
                MARGIN_BOTTOM / 2.0,
                hex(&label)
            ));
        }
    }
    assemble(layout.pages, PAGE_W, PAGE_H, doc.title().as_deref())
}

// ── Image pages ─────────────────────────────────────────

pub enum ImageData {
    /// A JPEG file embedded as-is (DCTDecode); `components` is 1 or 3.
    Jpeg { data: Vec<u8>, components: u8 },
    /// Raw 8-bit RGB with optional 8-bit alpha, Flate-compressed.
    Rgb {
        rgb: Vec<u8>,
        alpha: Option<Vec<u8>>,
    },
}

/// One A4 page per image, oriented to match, scaled to fit the margins but
/// never enlarged past its natural size at 96 DPI.
pub fn render_image(width: u32, height: u32, data: ImageData) -> Vec<u8> {
    let landscape = width > height;
    let (pw, ph) = if landscape {
        (PAGE_H, PAGE_W)
    } else {
        (PAGE_W, PAGE_H)
    };
    let margin = 36.0;
    let natural = (width as f32 * 0.75, height as f32 * 0.75);
    let scale = ((pw - 2.0 * margin) / natural.0)
        .min((ph - 2.0 * margin) / natural.1)
        .min(1.0);
    let (dw, dh) = (natural.0 * scale, natural.1 * scale);
    let (dx, dy) = ((pw - dw) / 2.0, (ph - dh) / 2.0);

    let mut b = Builder::new();
    let catalog = b.reserve();
    let pages_id = b.reserve();
    let image = match data {
        ImageData::Jpeg { data, components } => {
            let cs = if components == 1 {
                "DeviceGray"
            } else {
                "DeviceRGB"
            };
            b.stream(
                &format!(
                    "/Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /{cs} /BitsPerComponent 8 /Filter /DCTDecode"
                ),
                &data,
                false,
            )
        }
        ImageData::Rgb { rgb, alpha } => {
            let smask = alpha.map(|a| {
                b.stream(
                    &format!(
                        "/Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /DeviceGray /BitsPerComponent 8"
                    ),
                    &a,
                    true,
                )
            });
            let smask = smask.map_or(String::new(), |id| format!(" /SMask {id} 0 R"));
            b.stream(
                &format!(
                    "/Type /XObject /Subtype /Image /Width {width} /Height {height} /ColorSpace /DeviceRGB /BitsPerComponent 8{smask}"
                ),
                &rgb,
                true,
            )
        }
    };
    let ops = format!("q {dw:.2} 0 0 {dh:.2} {dx:.2} {dy:.2} cm /Im1 Do Q\n");
    let content = b.stream("", ops.as_bytes(), true);
    let page = b.add(format!(
        "<< /Type /Page /Parent {pages_id} 0 R /MediaBox [0 0 {pw:.2} {ph:.2}] /Contents {content} 0 R /Resources << /XObject << /Im1 {image} 0 R >> >> >>"
    ));
    b.set(
        pages_id,
        format!("<< /Type /Pages /Kids [{page} 0 R] /Count 1 >>").into_bytes(),
    );
    b.set(
        catalog,
        format!("<< /Type /Catalog /Pages {pages_id} 0 R >>").into_bytes(),
    );
    let info = b.add(format!("<< /Producer {} >>", text_string("Transfigure")));
    b.finish(catalog, info)
}

mod metrics {
    //! Advance widths (1/1000 em) for WinAnsi codes 32–255, taken from
    //! Liberation Sans, which is metric-compatible with Helvetica. Zero marks
    //! codes the encoding leaves undefined.
    pub const HELVETICA: [u16; 224] = [
        278, 278, 355, 556, 556, 889, 667, 191, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556,
        556, 556, 556, 556, 556, 556, 556, 556, 278, 278, 584, 584, 584, 556, 1015, 667, 667, 722,
        722, 667, 611, 778, 722, 278, 500, 667, 556, 833, 722, 778, 667, 778, 722, 667, 611, 722,
        667, 944, 667, 667, 611, 278, 278, 278, 469, 556, 333, 556, 556, 500, 556, 556, 278, 556,
        556, 222, 222, 500, 222, 833, 556, 556, 556, 556, 333, 500, 278, 556, 500, 722, 500, 500,
        500, 334, 260, 334, 584, 0, 556, 0, 222, 556, 333, 1000, 556, 556, 333, 1000, 667, 333,
        1000, 0, 611, 0, 0, 222, 222, 333, 333, 350, 556, 1000, 333, 1000, 500, 333, 944, 0, 500,
        667, 278, 333, 556, 556, 556, 556, 260, 556, 333, 737, 370, 556, 584, 333, 737, 552, 400,
        549, 333, 333, 333, 576, 537, 333, 333, 333, 365, 556, 834, 834, 834, 611, 667, 667, 667,
        667, 667, 667, 1000, 722, 667, 667, 667, 667, 278, 278, 278, 278, 722, 722, 778, 778, 778,
        778, 778, 584, 778, 722, 722, 722, 722, 667, 667, 611, 556, 556, 556, 556, 556, 556, 889,
        500, 556, 556, 556, 556, 278, 278, 278, 278, 556, 556, 556, 556, 556, 556, 556, 549, 611,
        556, 556, 556, 556, 500, 556, 500,
    ];
    pub const HELVETICA_BOLD: [u16; 224] = [
        278, 333, 474, 556, 556, 889, 722, 238, 333, 333, 389, 584, 278, 333, 278, 278, 556, 556,
        556, 556, 556, 556, 556, 556, 556, 556, 333, 333, 584, 584, 584, 611, 975, 722, 722, 722,
        722, 667, 611, 778, 722, 278, 556, 722, 611, 833, 722, 778, 667, 778, 722, 667, 611, 722,
        667, 944, 667, 667, 611, 333, 278, 333, 584, 556, 333, 556, 611, 556, 611, 556, 333, 611,
        611, 278, 278, 556, 278, 889, 611, 611, 611, 611, 389, 556, 333, 611, 556, 778, 556, 556,
        500, 389, 280, 389, 584, 0, 556, 0, 278, 556, 500, 1000, 556, 556, 333, 1000, 667, 333,
        1000, 0, 611, 0, 0, 278, 278, 500, 500, 350, 556, 1000, 333, 1000, 556, 333, 944, 0, 500,
        667, 278, 333, 556, 556, 556, 556, 280, 556, 333, 737, 370, 556, 584, 333, 737, 552, 400,
        549, 333, 333, 333, 576, 556, 333, 333, 333, 365, 556, 834, 834, 834, 611, 722, 722, 722,
        722, 722, 722, 1000, 722, 667, 667, 667, 667, 278, 278, 278, 278, 722, 722, 778, 778, 778,
        778, 778, 584, 778, 722, 722, 722, 722, 667, 667, 611, 556, 556, 556, 556, 556, 556, 889,
        556, 556, 556, 556, 556, 278, 278, 278, 278, 611, 611, 611, 611, 611, 611, 611, 549, 611,
        611, 611, 611, 611, 556, 611, 556,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn winansi_maps_typography_and_strips_accents() {
        assert_eq!(winansi("é—“x”"), vec![0xe9, 0x97, 0x93, b'x', 0x94]);
        assert_eq!(winansi("č ł"), b"c ?".to_vec());
        assert_eq!(winansi("a→b"), b"a->b".to_vec());
    }

    #[test]
    fn wrap_respects_width() {
        let text = "word ".repeat(200);
        let lines = wrap(&[Span::plain(text)], 200.0, 10.0, Style::default());
        assert!(lines.len() > 10);
        assert!(lines.iter().all(|l| l.width <= 200.01));
    }

    #[test]
    fn long_word_is_split() {
        let lines = wrap(
            &[Span::plain("x".repeat(500))],
            100.0,
            10.0,
            Style::default(),
        );
        assert!(lines.len() > 1);
        assert!(lines.iter().all(|l| l.width <= 100.01));
    }
}
