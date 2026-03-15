/// This example reads the project's own README.md and converts it to BBCode.
use markdown_bbcode::MdToBbcode;

fn main() {
    let readme = include_str!("../README.md");

    println!("--- README.md (Markdown) ---\n");
    println!("{readme}");

    let mut buf = Vec::new();
    MdToBbcode::new(readme, &mut buf).serialize().unwrap();
    let bbcode = String::from_utf8(buf).unwrap();

    println!("--- README.md (BBCode) ---\n");
    println!("{bbcode}");
}
