use super::{Block, Doc, Inlines, Span};

/// Plain text: blank lines separate paragraphs, single newlines are kept as
/// line breaks.
pub fn read(text: &str) -> Doc {
    let text = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut blocks = Vec::new();
    for para in text.split("\n\n") {
        let lines: Vec<&str> = para.lines().map(str::trim_end).collect();
        let lines: Vec<&str> = lines.iter().copied().skip_while(|l| l.is_empty()).collect();
        if lines.iter().all(|l| l.trim().is_empty()) {
            continue;
        }
        let mut inl: Inlines = Vec::new();
        for (i, line) in lines.iter().enumerate() {
            if i > 0 {
                inl.push(Span::line_break());
            }
            inl.push(Span::plain(*line));
        }
        blocks.push(Block::Para(inl));
    }
    Doc { blocks }
}

pub fn write(doc: &Doc) -> String {
    let mut s = render_blocks(&doc.blocks);
    let trimmed = s.trim_end().len();
    s.truncate(trimmed);
    s.push('\n');
    s
}

fn render_blocks(blocks: &[Block]) -> String {
    let mut s = String::new();
    for b in blocks {
        match b {
            Block::Heading(level, inl) => {
                let t = render_inlines(inl).replace('\n', " ");
                s.push_str(&t);
                s.push('\n');
                match level {
                    1 => s.push_str(&"=".repeat(t.chars().count().max(3))),
                    2 => s.push_str(&"-".repeat(t.chars().count().max(3))),
                    _ => {}
                }
                if *level <= 2 {
                    s.push('\n');
                }
                s.push('\n');
            }
            Block::Para(inl) => {
                s.push_str(&render_inlines(inl));
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
                    let pad = " ".repeat(marker.len());
                    let body = render_blocks(item);
                    for (j, line) in body.trim_end().lines().enumerate() {
                        s.push_str(if j == 0 { &marker } else { &pad });
                        s.push_str(line);
                        s.push('\n');
                    }
                }
                s.push('\n');
            }
            Block::Quote(inner) => {
                for line in render_blocks(inner).trim_end().lines() {
                    s.push_str("> ");
                    s.push_str(line);
                    s.push('\n');
                }
                s.push('\n');
            }
            Block::Code(code) => {
                for line in code.lines() {
                    s.push_str("    ");
                    s.push_str(line);
                    s.push('\n');
                }
                s.push('\n');
            }
            Block::Table(rows) => {
                s.push_str(&render_table(rows));
                s.push('\n');
            }
            Block::Rule => s.push_str("----------------------------------------\n\n"),
        }
    }
    s
}

/// Columns padded to line up in a monospace view.
fn render_table(rows: &[Vec<Inlines>]) -> String {
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|r| {
            r.iter()
                .map(|c| render_inlines(c).replace('\n', " "))
                .collect()
        })
        .collect();
    let cols = cells.iter().map(Vec::len).max().unwrap_or(0);
    let widths: Vec<usize> = (0..cols)
        .map(|i| {
            cells
                .iter()
                .map(|r| r.get(i).map_or(0, |c| c.chars().count()))
                .max()
                .unwrap_or(0)
        })
        .collect();
    let mut s = String::new();
    for (r, row) in cells.iter().enumerate() {
        let line: Vec<String> = (0..cols)
            .map(|i| {
                let c = row.get(i).map(String::as_str).unwrap_or("");
                format!("{c}{}", " ".repeat(widths[i] - c.chars().count()))
            })
            .collect();
        s.push_str(line.join("  ").trim_end());
        s.push('\n');
        if r == 0 && cells.len() > 1 {
            let rule: Vec<String> = widths.iter().map(|w| "-".repeat((*w).max(1))).collect();
            s.push_str(&rule.join("  "));
            s.push('\n');
        }
    }
    s
}

pub fn render_inlines(inl: &[Span]) -> String {
    let mut s = String::new();
    let mut i = 0;
    while i < inl.len() {
        if let Some(url) = &inl[i].link {
            let mut j = i;
            while j < inl.len() && inl[j].link.as_ref() == Some(url) {
                j += 1;
            }
            let label: String = inl[i..j].iter().map(|s| s.text.as_str()).collect();
            if label.trim() == url.as_str() || label.trim().is_empty() {
                s.push_str(url);
            } else {
                s.push_str(&format!("{label} ({url})"));
            }
            i = j;
        } else {
            s.push_str(&inl[i].text);
            i += 1;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paragraphs_and_line_breaks() {
        let doc = read("One\ntwo\n\n\nThree\r\n");
        assert_eq!(doc.blocks.len(), 2);
        assert_eq!(write(&doc), "One\ntwo\n\nThree\n");
    }

    #[test]
    fn table_columns_align() {
        let doc = Doc {
            blocks: vec![Block::Table(vec![
                vec![vec![Span::plain("Name")], vec![Span::plain("Value")]],
                vec![vec![Span::plain("alpha")], vec![Span::plain("1")]],
            ])],
        };
        assert_eq!(write(&doc), "Name   Value\n-----  -----\nalpha  1\n");
    }
}
