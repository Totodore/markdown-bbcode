# markdown-bbcode

A Rust library that converts Markdown to BBCode via [mdast](https://github.com/syntax-tree/mdast).

## Supported tags

| Markdown | BBCode |
|----------|--------|
| `**bold**` | `[b]bold[/b]` |
| `*italic*` | `[i]italic[/i]` |
| `~~strike~~` | `[s]strike[/s]` |
| `[text](url)` | `[url=url]text[/url]` |
| `![alt](url)` | `[img]url[/img]` |
| `` `code` `` | `[code]code[/code]` |
| ` ```lang ` | `[code=lang]...[/code]` |
| `# Heading` | `[size=7][b]Heading[/b][/size]` |
| `> quote` | `[quote]quote[/quote]` |
| `- item` | `[list][*]item[/list]` |
| `1. item` | `[list=1][*]item[/list]` |
| GFM tables | `[table][tr][th]...[/th][/tr][/table]` |
| `---` | `[hr]` |

## Usage

```rust
use markdown_bbcode::MdToBbcode;

let md = "**Hello** *world*!";
let mut buf = Vec::new();
MdToBbcode::new(md, &mut buf).serialize().unwrap();
let bbcode = String::from_utf8(buf).unwrap();
assert_eq!(bbcode, "[b]Hello[/b] [i]world[/i]!\n");
```
