use std::io::Write;

use markdown::{Constructs, ParseOptions, mdast::Node};

pub struct MdToBbcode<'a, W> {
    input: &'a str,
    writer: W,
    heading_size: [u8; 6],
}

impl<'a, W> MdToBbcode<'a, W> {
    pub fn new(input: &'a str, writer: W) -> Self {
        Self {
            input,
            writer,
            heading_size: default_heading_size(),
        }
    }
    pub fn with_heading_size(mut self, sizes: [u8; 6]) -> Self {
        self.heading_size = sizes;
        self
    }
}

impl<'a, W: Write> MdToBbcode<'a, W> {
    pub fn serialize(&mut self) -> Result<(), std::io::Error> {
        let options = ParseOptions {
            constructs: Constructs::gfm(),
            ..Default::default()
        };
        let root = markdown::to_mdast(self.input, &options).unwrap();
        self.next_node(&root)
    }

    fn next_node(&mut self, node: &Node) -> Result<(), std::io::Error> {
        match node {
            Node::Root(root) => root
                .children
                .iter()
                .try_for_each(|child| self.next_node(child)),
            Node::Text(text) => write!(self.writer, "{}", text.value),
            Node::Paragraph(paragraph) => {
                for child in &paragraph.children {
                    self.next_node(child)?;
                }
                writeln!(self.writer)
            }
            Node::Blockquote(blockquote) => {
                write!(self.writer, "[quote]")?;
                for child in &blockquote.children {
                    self.next_node(child)?;
                }
                write!(self.writer, "[/quote]")
            }
            Node::Heading(heading) => {
                let idx = std::cmp::max(heading.depth - 1, 5) as usize;
                let size = self.heading_size[idx];
                write!(self.writer, "[size={size}][b]")?;
                for child in &heading.children {
                    self.next_node(child)?;
                }
                writeln!(self.writer, "[/b][/size]")
            }
            Node::Strong(strong) => {
                write!(self.writer, "[b]")?;
                for child in &strong.children {
                    self.next_node(child)?;
                }
                write!(self.writer, "[/b]")
            }
            Node::Emphasis(emphasis) => {
                write!(self.writer, "[i]")?;
                for child in &emphasis.children {
                    self.next_node(child)?;
                }
                write!(self.writer, "[/i]")
            }
            Node::Delete(delete) => {
                write!(self.writer, "[s]")?;
                for child in &delete.children {
                    self.next_node(child)?;
                }
                write!(self.writer, "[/s]")
            }
            Node::Link(link) => {
                write!(self.writer, "[url={}]", link.url)?;
                for child in &link.children {
                    self.next_node(child)?;
                }
                write!(self.writer, "[/url]")
            }
            Node::Image(image) => {
                write!(self.writer, "[img]{}[/img]", image.url)
            }
            Node::InlineCode(inline_code) => {
                write!(self.writer, "[code]{}[/code]", inline_code.value)
            }
            Node::Code(code) => {
                if let Some(lang) = &code.lang {
                    write!(self.writer, "[code={lang}]")?;
                } else {
                    write!(self.writer, "[code]")?;
                }
                write!(self.writer, "{}", code.value)?;
                writeln!(self.writer, "[/code]")
            }
            Node::List(list) => {
                if list.ordered {
                    writeln!(self.writer, "[list=1]")?;
                } else {
                    writeln!(self.writer, "[list]")?;
                }
                for child in &list.children {
                    self.next_node(child)?;
                }
                writeln!(self.writer, "[/list]")
            }
            Node::ListItem(list_item) => {
                write!(self.writer, "[*]")?;
                for child in &list_item.children {
                    self.next_node(child)?;
                }
                Ok(())
            }
            Node::Table(table) => {
                write!(self.writer, "[table]")?;
                for (i, child) in table.children.iter().enumerate() {
                    self.next_table_row(child, i == 0)?;
                }
                writeln!(self.writer, "[/table]")
            }
            Node::TableRow(_) | Node::TableCell(_) => {
                // Handled by next_table_row / next_table_cell
                Ok(())
            }
            Node::Break(_) => writeln!(self.writer),
            Node::ThematicBreak(_) => writeln!(self.writer, "[hr]"),
            Node::Html(html) => write!(self.writer, "{}", html.value),
            Node::InlineMath(inline_math) => {
                write!(self.writer, "[math]{}[/math]", inline_math.value)
            }
            Node::Math(math) => {
                writeln!(self.writer, "[math]{}[/math]", math.value)
            }
            Node::FootnoteDefinition(footnote_definition) => {
                for child in &footnote_definition.children {
                    self.next_node(child)?;
                }
                Ok(())
            }
            Node::FootnoteReference(footnote_reference) => {
                write!(self.writer, "[{}]", footnote_reference.identifier)
            }
            Node::ImageReference(_) | Node::LinkReference(_) | Node::Definition(_) => {
                // References are resolved by mdast, these shouldn't appear in practice
                Ok(())
            }
            // MDX/frontmatter nodes — pass through or ignore
            Node::MdxJsxFlowElement(_)
            | Node::MdxjsEsm(_)
            | Node::MdxTextExpression(_)
            | Node::MdxJsxTextElement(_)
            | Node::MdxFlowExpression(_)
            | Node::Toml(_)
            | Node::Yaml(_) => Ok(()),
        }
    }
    fn next_table_row(&mut self, node: &Node, is_header: bool) -> Result<(), std::io::Error> {
        if let Node::TableRow(table_row) = node {
            write!(self.writer, "[tr]")?;
            for child in &table_row.children {
                self.next_table_cell(child, is_header)?;
            }
            write!(self.writer, "[/tr]")?;
        }
        Ok(())
    }

    fn next_table_cell(&mut self, node: &Node, is_header: bool) -> Result<(), std::io::Error> {
        if let Node::TableCell(table_cell) = node {
            let tag = if is_header { "th" } else { "td" };
            write!(self.writer, "[{tag}]")?;
            for child in &table_cell.children {
                self.next_node(child)?;
            }
            write!(self.writer, "[/{tag}]")?;
        }
        Ok(())
    }
}

fn default_heading_size() -> [u8; 6] {
    [
        18, // #
        14, // ##
        12, // ###
        10, // ####
        8,  // #####
        6,  // ######+
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convert(input: &str) -> String {
        let mut buf = Vec::new();
        MdToBbcode::new(input, &mut buf).serialize().unwrap();
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn plain_text() {
        assert_eq!(convert("hello"), "hello\n");
    }

    #[test]
    fn bold_and_italic() {
        assert_eq!(convert("**bold**"), "[b]bold[/b]\n");
        assert_eq!(convert("*italic*"), "[i]italic[/i]\n");
    }

    #[test]
    fn link() {
        assert_eq!(
            convert("[click](https://example.com)"),
            "[url=https://example.com]click[/url]\n"
        );
    }

    #[test]
    fn image() {
        assert_eq!(
            convert("![alt](https://img.png)"),
            "[img]https://img.png[/img]\n"
        );
    }

    #[test]
    fn code_block() {
        assert_eq!(
            convert("```rust\nfn main() {}\n```"),
            "[code=rust]fn main() {}[/code]\n"
        );
    }

    #[test]
    fn inline_code() {
        assert_eq!(convert("`code`"), "[code]code[/code]\n");
    }

    #[test]
    fn unordered_list() {
        assert_eq!(convert("- a\n- b\n"), "[list]\n[*]a\n[*]b\n[/list]\n");
    }

    #[test]
    fn ordered_list() {
        assert_eq!(convert("1. a\n2. b\n"), "[list=1]\n[*]a\n[*]b\n[/list]\n");
    }

    #[test]
    fn heading() {
        let result = convert("# Title");
        assert_eq!(result, "[size=7][b]Title[/b][/size]\n");
    }

    #[test]
    fn blockquote() {
        assert_eq!(convert("> quoted"), "[quote]quoted\n[/quote]");
    }

    #[test]
    fn strikethrough() {
        assert_eq!(convert("~~deleted~~"), "[s]deleted[/s]\n");
    }

    #[test]
    fn table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |";
        assert_eq!(
            convert(md),
            "[table][tr][th]A[/th][th]B[/th][/tr][tr][td]1[/td][td]2[/td][/tr][/table]\n"
        );
    }

    #[test]
    fn thematic_break() {
        assert_eq!(convert("---"), "[hr]\n");
    }
}
