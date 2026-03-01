use std::sync::Arc;

fn main() {
    let svg = include_str!("og.svg");

    let mut opt = resvg::usvg::Options::default();
    // Load system fonts so SVG text elements (title, tagline, pills) render correctly.
    Arc::make_mut(&mut opt.fontdb).load_system_fonts();

    let tree = resvg::usvg::Tree::from_str(svg, &opt).expect("failed to parse og.svg");

    let sz = tree.size().to_int_size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(sz.width(), sz.height()).expect("failed to create pixmap");

    resvg::render(
        &tree,
        resvg::usvg::Transform::default(),
        &mut pixmap.as_mut(),
    );

    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "crates/app/og.png".to_string());

    pixmap.save_png(&out).expect("failed to write PNG");
    eprintln!("✓  {out}  ({}×{})", sz.width(), sz.height());
}
