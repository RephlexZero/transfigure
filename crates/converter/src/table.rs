//! Tabular data: CSV, TSV, JSON records and spreadsheets share one model.

use std::io::Cursor;

use serde_json::{Map, Value};

#[derive(Clone, Debug, PartialEq)]
pub enum Cell {
    Empty,
    Text(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

impl Cell {
    fn to_text(&self) -> String {
        match self {
            Cell::Empty => String::new(),
            Cell::Text(s) => s.clone(),
            Cell::Int(n) => n.to_string(),
            Cell::Float(f) => format_float(*f),
            Cell::Bool(b) => b.to_string(),
        }
    }

    fn to_json(&self) -> Value {
        match self {
            Cell::Empty => Value::Null,
            Cell::Text(s) => Value::String(s.clone()),
            Cell::Int(n) => Value::from(*n),
            Cell::Float(f) => serde_json::Number::from_f64(*f)
                .map(Value::Number)
                .unwrap_or_else(|| Value::String(format_float(*f))),
            Cell::Bool(b) => Value::Bool(*b),
        }
    }
}

fn format_float(f: f64) -> String {
    if f.fract() == 0.0 && f.abs() < 1e15 {
        format!("{}", f as i64)
    } else {
        format!("{f}")
    }
}

/// Interpret a text field. Only canonical numbers become numbers, so values
/// like `007`, `+44`, `1,000` or `12.50` stay text and survive untouched.
fn typed(field: &str) -> Cell {
    if field.is_empty() {
        return Cell::Empty;
    }
    let digits = field.strip_prefix('-').unwrap_or(field);
    let canonical_int = !digits.is_empty()
        && digits.bytes().all(|b| b.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0'));
    if canonical_int && let Ok(n) = field.parse::<i64>() {
        return Cell::Int(n);
    }
    if let Some((int, frac)) = digits.split_once('.')
        && !int.is_empty()
        && int.bytes().all(|b| b.is_ascii_digit())
        && (int == "0" || !int.starts_with('0'))
        && !frac.is_empty()
        && frac.bytes().all(|b| b.is_ascii_digit())
        && !frac.ends_with('0')
        && let Ok(f) = field.parse::<f64>()
    {
        return Cell::Float(f);
    }
    Cell::Text(field.to_string())
}

pub type Rows = Vec<Vec<Cell>>;

// ── Readers ─────────────────────────────────────────────

pub fn read(input: &[u8], from: &str) -> Result<Rows, String> {
    match from {
        "csv" => read_delimited(input, None),
        "tsv" => read_delimited(input, Some(b'\t')),
        "xlsx" | "xls" | "xlsm" | "xlsb" | "ods" => read_spreadsheet(input),
        "json" => read_json(input),
        _ => Err(format!("Unsupported table format: {from}")),
    }
}

/// Pick the delimiter used in the first line, ignoring quoted text: CSV
/// exported by Excel in much of Europe uses semicolons.
fn sniff_delimiter(text: &str) -> u8 {
    let first = text.lines().next().unwrap_or("");
    let mut counts = [0usize; 4];
    let cands = [b',', b';', b'\t', b'|'];
    let mut quoted = false;
    for b in first.bytes() {
        if b == b'"' {
            quoted = !quoted;
        } else if !quoted && let Some(i) = cands.iter().position(|c| *c == b) {
            counts[i] += 1;
        }
    }
    let (best, n) = counts
        .iter()
        .enumerate()
        .max_by_key(|(i, n)| (**n, std::cmp::Reverse(*i)))
        .expect("non-empty");
    if *n == 0 { b',' } else { cands[best] }
}

fn read_delimited(input: &[u8], delimiter: Option<u8>) -> Result<Rows, String> {
    let text = String::from_utf8_lossy(input);
    let text = text.strip_prefix('\u{feff}').unwrap_or(&text);
    let delim = delimiter.unwrap_or_else(|| sniff_delimiter(text));
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delim)
        .has_headers(false)
        .flexible(true)
        .quoting(delim != b'\t' || text.contains('"'))
        .from_reader(text.as_bytes());
    let mut rows = Vec::new();
    for rec in rdr.records() {
        let rec = rec.map_err(|e| format!("Could not parse row: {e}"))?;
        rows.push(rec.iter().map(typed).collect());
    }
    Ok(rows)
}

fn read_spreadsheet(input: &[u8]) -> Result<Rows, String> {
    use calamine::{Data, Reader};
    let mut book = calamine::open_workbook_auto_from_rs(Cursor::new(input))
        .map_err(|e| format!("Could not open spreadsheet: {e}"))?;
    let sheets = book.worksheets();
    // The first sheet with any content.
    let (_, range) = sheets
        .into_iter()
        .find(|(_, r)| r.rows().any(|row| row.iter().any(|c| *c != Data::Empty)))
        .ok_or_else(|| "The spreadsheet is empty".to_string())?;
    let mut rows: Rows = range
        .rows()
        .map(|row| {
            row.iter()
                .map(|c| match c {
                    Data::Empty => Cell::Empty,
                    Data::String(s) => Cell::Text(s.clone()),
                    Data::Int(n) => Cell::Int(*n),
                    Data::Float(f) if f.fract() == 0.0 && f.abs() < 9e15 => Cell::Int(*f as i64),
                    Data::Float(f) => Cell::Float(*f),
                    Data::Bool(b) => Cell::Bool(*b),
                    Data::DateTime(d) => Cell::Text(excel_date(d.as_f64(), d.is_duration())),
                    Data::DateTimeIso(s) | Data::DurationIso(s) => Cell::Text(s.clone()),
                    Data::Error(e) => Cell::Text(format!("#{e:?}")),
                })
                .collect()
        })
        .collect();
    trim(&mut rows);
    Ok(rows)
}

/// Excel serial date (days since 1899-12-30) as ISO 8601.
fn excel_date(serial: f64, duration: bool) -> String {
    let secs_total = (serial.fract().abs() * 86_400.0).round() as i64;
    let (h, m, s) = (secs_total / 3600, secs_total / 60 % 60, secs_total % 60);
    if duration {
        let h = serial.trunc() as i64 * 24 + h;
        return format!("{h}:{m:02}:{s:02}");
    }
    // Civil-from-days (Howard Hinnant), shifted to the Excel epoch.
    let days = serial.floor() as i64 - 25_569 + 719_468;
    let era = days.div_euclid(146_097);
    let doe = days.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(mo <= 2);
    if serial < 1.0 {
        format!("{h:02}:{m:02}:{s:02}")
    } else if secs_total == 0 {
        format!("{y:04}-{mo:02}-{d:02}")
    } else {
        format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}")
    }
}

/// Drop trailing empty rows and columns.
fn trim(rows: &mut Rows) {
    while rows
        .last()
        .is_some_and(|r| r.iter().all(|c| *c == Cell::Empty))
    {
        rows.pop();
    }
    let width = rows
        .iter()
        .map(|r| {
            r.iter()
                .rposition(|c| *c != Cell::Empty)
                .map_or(0, |i| i + 1)
        })
        .max()
        .unwrap_or(0);
    for r in rows.iter_mut() {
        r.truncate(width);
    }
}

fn json_cell(v: &Value) -> Cell {
    match v {
        Value::Null => Cell::Empty,
        Value::Bool(b) => Cell::Bool(*b),
        Value::Number(n) => n
            .as_i64()
            .map(Cell::Int)
            .or_else(|| n.as_f64().map(Cell::Float))
            .unwrap_or_else(|| Cell::Text(n.to_string())),
        Value::String(s) => Cell::Text(s.clone()),
        Value::Array(a) if a.iter().all(|x| !x.is_object() && !x.is_array()) => Cell::Text(
            a.iter()
                .map(|x| match x {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                })
                .collect::<Vec<_>>()
                .join("; "),
        ),
        other => Cell::Text(other.to_string()),
    }
}

/// Flatten nested objects into dotted column names: `{"a":{"b":1}}` → `a.b`.
fn flatten(prefix: &str, obj: &Map<String, Value>, out: &mut Vec<(String, Value)>) {
    for (k, v) in obj {
        let key = if prefix.is_empty() {
            k.clone()
        } else {
            format!("{prefix}.{k}")
        };
        match v {
            Value::Object(inner) if !inner.is_empty() => flatten(&key, inner, out),
            _ => out.push((key, v.clone())),
        }
    }
}

fn read_json(input: &[u8]) -> Result<Rows, String> {
    let value: Value = serde_json::from_slice(input).map_err(|e| format!("Invalid JSON: {e}"))?;
    let items: Vec<Value> = match value {
        Value::Array(a) => a,
        Value::Object(ref o) => {
            // `{"data": [...]}`-style wrappers: use the single array inside.
            let arrays: Vec<&Value> = o.values().filter(|v| v.is_array()).collect();
            if o.len() == 1 && arrays.len() == 1 {
                arrays[0].as_array().cloned().unwrap_or_default()
            } else {
                vec![value]
            }
        }
        other => vec![other],
    };
    if items.iter().all(Value::is_array) {
        return Ok(items
            .iter()
            .map(|r| {
                r.as_array()
                    .expect("checked")
                    .iter()
                    .map(json_cell)
                    .collect()
            })
            .collect());
    }
    let mut header: Vec<String> = Vec::new();
    let mut records = Vec::new();
    for item in &items {
        let mut flat = Vec::new();
        match item {
            Value::Object(o) => flatten("", o, &mut flat),
            other => flat.push(("value".to_string(), other.clone())),
        }
        for (k, _) in &flat {
            if !header.contains(k) {
                header.push(k.clone());
            }
        }
        records.push(flat);
    }
    let mut rows: Rows = vec![header.iter().map(|h| Cell::Text(h.clone())).collect()];
    for rec in records {
        rows.push(
            header
                .iter()
                .map(|h| {
                    rec.iter()
                        .find(|(k, _)| k == h)
                        .map_or(Cell::Empty, |(_, v)| json_cell(v))
                })
                .collect(),
        );
    }
    Ok(rows)
}

// ── Writers ─────────────────────────────────────────────

pub fn write(rows: &Rows, to: &str) -> Result<Vec<u8>, String> {
    match to {
        "csv" => write_delimited(rows, b','),
        "tsv" => write_delimited(rows, b'\t'),
        "json" => write_json(rows),
        "xlsx" => write_xlsx(rows),
        _ => Err(format!("Unsupported table output: {to}")),
    }
}

fn write_delimited(rows: &Rows, delim: u8) -> Result<Vec<u8>, String> {
    let mut wtr = csv::WriterBuilder::new()
        .delimiter(delim)
        .flexible(true)
        .from_writer(Vec::new());
    for row in rows {
        let fields: Vec<String> = row
            .iter()
            .map(|c| {
                let t = c.to_text();
                if delim == b'\t' {
                    // TSV has no quoting convention; keep one record per line.
                    t.replace(['\t', '\n', '\r'], " ")
                } else {
                    t
                }
            })
            .collect();
        wtr.write_record(&fields)
            .map_err(|e| format!("Write error: {e}"))?;
    }
    wtr.into_inner().map_err(|e| format!("Write error: {e}"))
}

fn header_names(rows: &Rows) -> Vec<String> {
    let mut names: Vec<String> = Vec::new();
    let first = rows.first().cloned().unwrap_or_default();
    let width = rows.iter().map(Vec::len).max().unwrap_or(0);
    for i in 0..width {
        let base = first
            .get(i)
            .map(Cell::to_text)
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| format!("column_{}", i + 1));
        let mut name = base.clone();
        let mut n = 2;
        while names.contains(&name) {
            name = format!("{base}_{n}");
            n += 1;
        }
        names.push(name);
    }
    names
}

fn write_json(rows: &Rows) -> Result<Vec<u8>, String> {
    let names = header_names(rows);
    let records: Vec<Value> = rows
        .iter()
        .skip(1)
        .map(|row| {
            let mut m = Map::new();
            for (i, name) in names.iter().enumerate() {
                m.insert(name.clone(), row.get(i).map_or(Value::Null, Cell::to_json));
            }
            Value::Object(m)
        })
        .collect();
    let mut out = serde_json::to_vec_pretty(&records).map_err(|e| e.to_string())?;
    out.push(b'\n');
    Ok(out)
}

fn write_xlsx(rows: &Rows) -> Result<Vec<u8>, String> {
    use rust_xlsxwriter::{Format, Workbook};
    let err = |e: rust_xlsxwriter::XlsxError| format!("XLSX error: {e}");
    let mut book = Workbook::new();
    let bold = Format::new().set_bold();
    let sheet = book.add_worksheet();
    sheet.set_name("Sheet1").map_err(err)?;
    for (r, row) in rows.iter().enumerate() {
        let r32 = u32::try_from(r).map_err(|_| "Too many rows for XLSX".to_string())?;
        for (c, cell) in row.iter().enumerate() {
            let c16 = u16::try_from(c).map_err(|_| "Too many columns for XLSX".to_string())?;
            match cell {
                Cell::Empty => {}
                Cell::Text(s) if r == 0 => {
                    sheet
                        .write_string_with_format(r32, c16, s, &bold)
                        .map_err(err)?;
                }
                Cell::Text(s) => {
                    sheet.write_string(r32, c16, s).map_err(err)?;
                }
                Cell::Int(n) => {
                    sheet.write_number(r32, c16, *n as f64).map_err(err)?;
                }
                Cell::Float(f) => {
                    sheet.write_number(r32, c16, *f).map_err(err)?;
                }
                Cell::Bool(b) => {
                    sheet.write_boolean(r32, c16, *b).map_err(err)?;
                }
            }
        }
    }
    if rows.len() > 1 {
        sheet.set_freeze_panes(1, 0).map_err(err)?;
    }
    sheet.autofit();
    book.save_to_buffer().map_err(err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn typing_keeps_identifiers_as_text() {
        assert_eq!(typed("007"), Cell::Text("007".into()));
        assert_eq!(typed("+44"), Cell::Text("+44".into()));
        assert_eq!(typed("12.50"), Cell::Text("12.50".into()));
        assert_eq!(typed("1,000"), Cell::Text("1,000".into()));
        assert_eq!(typed("42"), Cell::Int(42));
        assert_eq!(typed("-3"), Cell::Int(-3));
        assert_eq!(typed("0.5"), Cell::Float(0.5));
        assert_eq!(typed("0"), Cell::Int(0));
    }

    #[test]
    fn semicolon_csv_is_detected() {
        let rows = read(b"a;b\n1;\"x;y\"\n", "csv").unwrap();
        assert_eq!(rows[1], vec![Cell::Int(1), Cell::Text("x;y".into())]);
    }

    #[test]
    fn nested_json_flattens_to_columns() {
        let rows = read(br#"[{"a":1,"b":{"c":"x"}},{"a":2,"d":[1,2]}]"#, "json").unwrap();
        assert_eq!(
            rows[0],
            vec![
                Cell::Text("a".into()),
                Cell::Text("b.c".into()),
                Cell::Text("d".into())
            ]
        );
        assert_eq!(rows[2][2], Cell::Text("1; 2".into()));
    }

    #[test]
    fn excel_dates() {
        assert_eq!(excel_date(45_000.0, false), "2023-03-15");
        assert_eq!(excel_date(45_000.5, false), "2023-03-15T12:00:00");
        assert_eq!(excel_date(1.5, true), "36:00:00");
    }

    #[test]
    fn csv_json_roundtrip_preserves_column_order() {
        let rows = read(b"zeta,alpha\n1,x\n", "csv").unwrap();
        let json = String::from_utf8(write(&rows, "json").unwrap()).unwrap();
        assert!(json.find("zeta").unwrap() < json.find("alpha").unwrap());
    }
}
