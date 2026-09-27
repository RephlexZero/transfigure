//! Text extraction from PDF. PDFs store positioned glyphs, not paragraphs, so
//! this recovers readable paragraphs: lines within a block are joined and
//! blank lines separate blocks.

use super::{Block, Doc, Span};

pub fn read(input: &[u8]) -> Result<Doc, String> {
    let doc = lopdf::Document::load_mem(input).map_err(|e| format!("Failed to load PDF: {e}"))?;
    let mut pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    pages.sort_unstable();

    let mut blocks = Vec::new();
    for page in pages {
        let text = doc.extract_text(&[page]).unwrap_or_default();
        let mut para = String::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() {
                flush(&mut para, &mut blocks);
                continue;
            }
            if para.ends_with('-') && !para.ends_with(" -") {
                // Rejoin a word hyphenated across lines.
                para.pop();
            } else if !para.is_empty() {
                para.push(' ');
            }
            para.push_str(line);
        }
        flush(&mut para, &mut blocks);
    }
    if blocks.is_empty() {
        return Err("No text found in this PDF (it may be scanned images)".into());
    }
    Ok(Doc { blocks })
}

fn flush(para: &mut String, blocks: &mut Vec<Block>) {
    let t = tidy(&para.split_whitespace().collect::<Vec<_>>().join(" "));
    if !t.is_empty() {
        blocks.push(Block::Para(vec![Span::plain(t)]));
    }
    para.clear();
}

/// Text runs come out of PDFs separated by spaces, which strands punctuation
/// that followed a styled word ("bold , italic"). Reattach it.
fn tidy(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c == ' ' {
            let next = chars.get(i + 1).copied();
            let prev = i.checked_sub(1).map(|j| chars[j]);
            if matches!(next, Some(',' | '.' | ';' | ':' | '!' | '?' | ')' | ']'))
                || matches!(prev, Some('(' | '['))
            {
                continue;
            }
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    #[test]
    fn tidy_reattaches_punctuation() {
        assert_eq!(
            super::tidy("bold , italic ( x ) end ."),
            "bold, italic (x) end."
        );
    }
}
