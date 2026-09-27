//! Run one conversion from the command line, for debugging:
//!
//!     cargo run -p converter --example convert -- in.heic heic jpg out.jpg

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let [_, input, from, to, output] = args.as_slice() else {
        eprintln!("usage: convert <input> <from> <to> <output>");
        std::process::exit(2);
    };
    let data = std::fs::read(input).expect("read input");
    let config = serde_json::json!({ "from": from, "to": to }).to_string();
    match converter::convert(&data, &config) {
        Ok(out) => std::fs::write(output, out).expect("write output"),
        Err(e) => {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
    }
}
