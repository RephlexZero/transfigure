use super::{Block, Doc, Span, markdown};

/// HTML → Markdown via htmd, which copes with arbitrary real-world markup.
pub fn to_markdown(html: &str) -> String {
    let converter = htmd::HtmlToMarkdown::builder()
        .options(htmd::options::Options {
            bullet_list_marker: htmd::options::BulletListMarker::Dash,
            hr_style: htmd::options::HrStyle::Dashes,
            ul_bullet_spacing: 1,
            ol_number_spacing: 1,
            ..Default::default()
        })
        .skip_tags(vec![
            "script", "style", "noscript", "template", "head", "iframe", "svg", "canvas",
        ])
        .build();
    converter
        .convert(html)
        .unwrap_or_default()
        .trim()
        .to_string()
}

pub fn read(html: &str) -> Doc {
    markdown::read(&to_markdown(html))
}

// ── Writer ──────────────────────────────────────────────

/// Wrap body markup in a standalone, readable HTML page. The charset
/// declaration matters: without it, a local file with accents renders as
/// mojibake in most browsers.
pub fn html_document(title: Option<&str>, body: &str) -> String {
    let title = escape(title.unwrap_or("Document"));
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>
  body {{ max-width: 46rem; margin: 2.5rem auto; padding: 0 1.25rem; font: 16px/1.6 system-ui, -apple-system, "Segoe UI", Roboto, sans-serif; color: #1c1917; }}
  h1, h2, h3, h4 {{ line-height: 1.25; margin: 1.6em 0 0.5em; }}
  pre, code {{ font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace; font-size: 0.9em; }}
  pre {{ background: #f5f4f1; padding: 0.9rem 1rem; border-radius: 6px; overflow-x: auto; }}
  :not(pre) > code {{ background: #f5f4f1; padding: 0.1em 0.3em; border-radius: 4px; }}
  blockquote {{ margin: 1em 0; padding-left: 1em; border-left: 3px solid #d6d3d1; color: #57534e; }}
  table {{ border-collapse: collapse; margin: 1em 0; }}
  th, td {{ border: 1px solid #d6d3d1; padding: 0.35em 0.7em; text-align: left; }}
  th {{ background: #f5f4f1; }}
  img {{ max-width: 100%; }}
  hr {{ border: 0; border-top: 1px solid #d6d3d1; margin: 2em 0; }}
</style>
</head>
<body>
{body}
</body>
</html>
"#
    )
}

pub fn write(doc: &Doc) -> String {
    let title = doc.title();
    html_document(title.as_deref(), &render_blocks(&doc.blocks))
}

fn render_blocks(blocks: &[Block]) -> String {
    let mut s = String::new();
    for b in blocks {
        match b {
            Block::Heading(level, inl) => {
                let l = (*level).clamp(1, 6);
                s.push_str(&format!("<h{l}>{}</h{l}>\n", render_inlines(inl)));
            }
            Block::Para(inl) => s.push_str(&format!("<p>{}</p>\n", render_inlines(inl))),
            Block::List {
                ordered,
                start,
                items,
            } => {
                if *ordered {
                    if *start == 1 {
                        s.push_str("<ol>\n");
                    } else {
                        s.push_str(&format!("<ol start=\"{start}\">\n"));
                    }
                } else {
                    s.push_str("<ul>\n");
                }
                for item in items {
                    // A single paragraph renders inline, like a tight list.
                    match item.as_slice() {
                        [Block::Para(inl)] => {
                            s.push_str(&format!("<li>{}</li>\n", render_inlines(inl)))
                        }
                        _ => s.push_str(&format!("<li>{}</li>\n", render_blocks(item).trim_end())),
                    }
                }
                s.push_str(if *ordered { "</ol>\n" } else { "</ul>\n" });
            }
            Block::Quote(inner) => s.push_str(&format!(
                "<blockquote>\n{}</blockquote>\n",
                render_blocks(inner)
            )),
            Block::Code(code) => s.push_str(&format!("<pre><code>{}</code></pre>\n", escape(code))),
            Block::Table(rows) => {
                s.push_str("<table>\n");
                for (r, row) in rows.iter().enumerate() {
                    let tag = if r == 0 { "th" } else { "td" };
                    s.push_str("<tr>");
                    for cell in row {
                        s.push_str(&format!("<{tag}>{}</{tag}>", render_inlines(cell)));
                    }
                    s.push_str("</tr>\n");
                }
                s.push_str("</table>\n");
            }
            Block::Rule => s.push_str("<hr>\n"),
        }
    }
    s
}

fn render_inlines(inl: &[Span]) -> String {
    let mut s = String::new();
    let mut i = 0;
    while i < inl.len() {
        if let Some(url) = &inl[i].link {
            let mut j = i;
            while j < inl.len() && inl[j].link.as_ref() == Some(url) {
                j += 1;
            }
            let label: String = inl[i..j].iter().map(render_span).collect();
            s.push_str(&format!("<a href=\"{}\">{label}</a>", escape_attr(url)));
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
        return "<br>\n".into();
    }
    let mut t = escape(&span.text);
    let st = span.style;
    if st.code {
        t = format!("<code>{t}</code>");
    }
    if st.strike {
        t = format!("<s>{t}</s>");
    }
    if st.italic {
        t = format!("<em>{t}</em>");
    }
    if st.bold {
        t = format!("<strong>{t}</strong>");
    }
    t
}

pub fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attr(s: &str) -> String {
    escape(s).replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripts_and_styles_are_dropped() {
        let md = to_markdown(
            "<html><head><style>p{}</style><script>alert(1)</script></head><body><p>Hi</p><script>x()</script></body></html>",
        );
        assert_eq!(md, "Hi");
    }

    #[test]
    fn document_declares_charset() {
        let html = write(&Doc {
            blocks: vec![Block::Para(vec![Span::plain("Café <b>")])],
        });
        assert!(html.contains("<meta charset=\"utf-8\">"));
        assert!(html.contains("<p>Café &lt;b&gt;</p>"));
    }
}
