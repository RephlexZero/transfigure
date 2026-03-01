pub fn csv_to_json(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut rdr = csv::Reader::from_reader(input);
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| format!("Failed to read CSV headers: {e}"))?
        .iter()
        .map(|h| h.to_string())
        .collect();

    let mut records: Vec<serde_json::Value> = Vec::new();
    for result in rdr.records() {
        let record = result.map_err(|e| format!("CSV parse error: {e}"))?;
        let mut map = serde_json::Map::new();
        for (i, field) in record.iter().enumerate() {
            let key = headers.get(i).cloned().unwrap_or_else(|| format!("col{i}"));
            let value = if let Ok(n) = field.parse::<i64>() {
                serde_json::Value::Number(n.into())
            } else if let Ok(n) = field.parse::<f64>() {
                serde_json::Number::from_f64(n)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::String(field.to_string()))
            } else {
                serde_json::Value::String(field.to_string())
            };
            map.insert(key, value);
        }
        records.push(serde_json::Value::Object(map));
    }

    serde_json::to_string_pretty(&records)
        .map(|s| s.into_bytes())
        .map_err(|e| format!("JSON serialization error: {e}"))
}

pub fn json_to_csv(input: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    let data: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("Invalid JSON: {e}"))?;

    let array = match &data {
        serde_json::Value::Array(a) => a,
        _ => return Err("JSON must be an array of objects".into()),
    };

    if array.is_empty() {
        return Ok(Vec::new());
    }

    let mut headers: Vec<String> = Vec::new();
    for item in array {
        if let serde_json::Value::Object(map) = item {
            for key in map.keys() {
                if !headers.contains(key) {
                    headers.push(key.clone());
                }
            }
        }
    }

    let mut wtr = csv::Writer::from_writer(Vec::new());
    wtr.write_record(&headers)
        .map_err(|e| format!("CSV write error: {e}"))?;

    for item in array {
        if let serde_json::Value::Object(map) = item {
            let row: Vec<String> = headers
                .iter()
                .map(|h| match map.get(h) {
                    Some(serde_json::Value::String(s)) => s.clone(),
                    Some(v) => v.to_string(),
                    None => String::new(),
                })
                .collect();
            wtr.write_record(&row)
                .map_err(|e| format!("CSV write error: {e}"))?;
        }
    }

    wtr.into_inner()
        .map_err(|e| format!("CSV flush error: {e}"))
}

pub fn csv_to_tsv(input: &[u8]) -> Result<Vec<u8>, String> {
    let mut rdr = csv::Reader::from_reader(input);
    let mut output = String::new();

    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| format!("Failed to read CSV headers: {e}"))?
        .iter()
        .map(|h| h.to_string())
        .collect();
    output.push_str(&headers.join("\t"));
    output.push('\n');

    for result in rdr.records() {
        let record = result.map_err(|e| format!("CSV parse error: {e}"))?;
        let fields: Vec<&str> = record.iter().collect();
        output.push_str(&fields.join("\t"));
        output.push('\n');
    }

    Ok(output.into_bytes())
}

pub fn tsv_to_csv(input: &[u8]) -> Result<Vec<u8>, String> {
    let text = std::str::from_utf8(input).map_err(|e| format!("Invalid UTF-8: {e}"))?;
    let mut wtr = csv::Writer::from_writer(Vec::new());

    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        wtr.write_record(&fields)
            .map_err(|e| format!("CSV write error: {e}"))?;
    }

    wtr.into_inner()
        .map_err(|e| format!("CSV flush error: {e}"))
}
