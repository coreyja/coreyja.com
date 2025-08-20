use markdown::mdast::{Node, Root};

pub trait IntoPlainText {
    fn plain_text(&self) -> String;
}

impl IntoPlainText for Node {
    fn plain_text(&self) -> String {
        fn blank() -> String {
            String::new()
        }

        fn surround_with(surround_token: &str, inner: &str) -> String {
            format!("{surround_token}{inner}{surround_token}")
        }

        match self {
            Node::Root(x) => x.children.plain_text(),
            Node::Blockquote(x) => x.children.plain_text(),
            Node::FootnoteDefinition(x) => x.children.plain_text(),
            Node::List(x) => x.children.plain_text(),
            Node::InlineCode(x) => x.value.clone(),
            Node::InlineMath(x) => x.value.clone(),
            Node::Delete(x) => surround_with("~", &x.children.plain_text()),
            Node::Emphasis(x) => surround_with("*", &x.children.plain_text()),
            Node::Link(x) => x.children.plain_text(),
            Node::Strong(x) => surround_with("*", &x.children.plain_text()),
            Node::Text(x) => x.value.clone(),
            Node::Code(x) => surround_with("\n```\n", &x.value),
            Node::Math(x) => x.value.clone(),
            Node::Heading(x) => x.children.plain_text(),
            Node::Table(x) => x.children.plain_text(),
            Node::TableRow(x) => x.children.plain_text(),
            Node::TableCell(x) => x.children.plain_text(),
            Node::ListItem(x) => x.children.iter().map(IntoPlainText::plain_text).collect(),
            Node::Definition(_) => blank(),
            Node::Paragraph(x) => x.children.plain_text(),
            Node::MdxJsxFlowElement(_)
            | Node::MdxjsEsm(_)
            | Node::Toml(_)
            | Node::Yaml(_)
            | Node::Break(_)
            | Node::MdxTextExpression(_)
            | Node::FootnoteReference(_)
            | Node::Html(_)
            | Node::Image(_)
            | Node::ImageReference(_)
            | Node::MdxJsxTextElement(_)
            | Node::LinkReference(_)
            | Node::MdxFlowExpression(_)
            | Node::ThematicBreak(_) => String::new(),
        }
    }
}

impl IntoPlainText for Vec<Node> {
    fn plain_text(&self) -> String {
        self.iter()
            .map(IntoPlainText::plain_text)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

impl IntoPlainText for Root {
    fn plain_text(&self) -> String {
        self.children.plain_text().trim().to_owned()
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use crate::{plain::IntoPlainText, MarkdownAst};

    #[test]
    fn simple() {
        let original = "Hello, world!";
        let parsed = MarkdownAst::from_str(original).unwrap();
        let plain = parsed.0.plain_text();

        assert_eq!(plain, original);
    }

    #[test]
    #[ignore = "The newlines around paragraphs aren't quite right"]
    fn heading() {
        let original = "# Hello, world!

This is a test.

More words
these are together
I think

# Another heading";
        let parsed = MarkdownAst::from_str(original).unwrap();
        let plain = parsed.0.plain_text();

        assert_eq!(
            plain,
            "Hello, world!

This is a test."
        );
    }

    #[test]
    fn code() {
        let original = "```
let x = \"test\";
```";
        let parsed = MarkdownAst::from_str(original).unwrap();
        let plain = parsed.0.plain_text();

        assert_eq!(
            plain,
            "```
let x = \"test\";
```"
        );
    }

    #[test]
    #[ignore = "We dont add the language to the code block yet"]
    fn code_with_lang() {
        let original = "```rust
let x = \"test\";
```";
        let parsed = MarkdownAst::from_str(original).unwrap();
        let plain = parsed.0.plain_text();

        assert_eq!(
            plain,
            "```rust
let x = \"test\";
```"
        );
    }

    #[test]
    #[ignore = "We don't add dashes before the list items yet"]
    fn ul_list() {
        let original = "- Hello, world!
- This is a test.";
        let parsed = MarkdownAst::from_str(original).unwrap();
        let plain = parsed.0.plain_text();

        assert_eq!(plain, original);
    }

    #[test]
    #[ignore = "We don't add numbers before the list items yet"]
    fn ol_list() {
        let original = "0. Hello, world!
0. This is a test.";
        let parsed = MarkdownAst::from_str(original).unwrap();
        let plain = parsed.0.plain_text();

        assert_eq!(
            plain,
            "1. Hello, world!
2. This is a test."
        );
    }
}
