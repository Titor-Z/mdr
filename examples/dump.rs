/// Example: render a markdown file and print lines as plain text (for debugging).
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).map(|s| s.as_str()).unwrap_or("test.md");
    let content = std::fs::read_to_string(path).expect("cannot read file");
    let lines = mdr::render::render(&content).expect("render failed");
    for line in &lines {
        println!("{}", line);
    }
}
