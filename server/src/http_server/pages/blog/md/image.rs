use markdown::mdast::{Node, Paragraph, Root};

pub(crate) trait FindCoverPhoto {
    fn cover_photo(&self) -> Option<String>;
}

impl FindCoverPhoto for Root {
    fn cover_photo(&self) -> Option<String> {
        self.children.first().and_then(FindCoverPhoto::cover_photo)
    }
}

impl FindCoverPhoto for Paragraph {
    fn cover_photo(&self) -> Option<String> {
        self.children.first().and_then(FindCoverPhoto::cover_photo)
    }
}

impl FindCoverPhoto for Node {
    fn cover_photo(&self) -> Option<String> {
        match self {
            Node::Root(r) => r.cover_photo(),
            Node::Paragraph(p) => p.cover_photo(),
            Node::Image(i) => Some(i.url.clone()),
            _ => None,
        }
    }
}
