use super::Post;

pub trait Title {
    fn title(&self) -> &str;
}

impl<FrontMatter> Title for Post<FrontMatter>
where
    FrontMatter: Title,
{
    fn title(&self) -> &str {
        self.frontmatter.title()
    }
}
