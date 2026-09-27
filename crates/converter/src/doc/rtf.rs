//! RTF reading via rtf-parser: paragraphs, bold, italic, and headings
//! inferred from font size (RTF rarely marks headings semantically).

use std::collections::HashMap;

use rtf_parser::document::RtfDocument;

use super::{Block, Doc, Inlines, Style, normalize_inlines, push_text};

pub fn read(input: &[u8]) -> Result<Doc, String> {
    let text = String::from_utf8_lossy(input);
    let rtf =
        RtfDocument::try_from(text.as_ref()).map_err(|e| format!("Failed to parse RTF: {e}"))?;

    // Split style blocks into paragraphs, remembering each run's font size.
    let mut paras: Vec<Vec<(String, Style, u16)>> = vec![Vec::new()];
    for block in &rtf.body {
        let st = Style {
            bold: block.painter.bold,
            italic: block.painter.italic,
            strike: block.painter.strike,
            code: false,
        };
        let size = block.painter.font_size;
        for (i, part) in block.text.split('\n').enumerate() {
            if i > 0 {
                paras.push(Vec::new());
            }
            if !part.is_empty() {
                paras
                    .last_mut()
                    .expect("non-empty")
                    .push((part.to_string(), st, size));
            }
        }
    }

    // Body size: the size carrying the most characters.
    let mut by_size: HashMap<u16, usize> = HashMap::new();
    for (t, _, size) in paras.iter().flatten() {
        *by_size.entry(*size).or_default() += t.chars().count();
    }
    let body = by_size
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map_or(12, |(s, _)| s)
        .max(1) as f32;

    let mut blocks = Vec::new();
    for para in paras {
        let mut inl: Inlines = Vec::new();
        for (t, st, _) in &para {
            push_text(&mut inl, t, *st, None);
        }
        let inl = normalize_inlines(inl);
        if inl.is_empty() {
            continue;
        }
        let min_size = para.iter().map(|(_, _, s)| *s).min().unwrap_or(0) as f32;
        let ratio = min_size / body;
        let short = inl.iter().map(|s| s.text.len()).sum::<usize>() < 200;
        if short && ratio >= 1.7 {
            blocks.push(Block::Heading(1, inl));
        } else if short && ratio >= 1.35 {
            blocks.push(Block::Heading(2, inl));
        } else if short && ratio >= 1.15 {
            blocks.push(Block::Heading(3, inl));
        } else {
            blocks.push(Block::Para(inl));
        }
    }
    Ok(Doc { blocks })
}
