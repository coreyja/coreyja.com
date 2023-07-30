use super::Post;

pub trait PostedOn {
    fn posted_on(&self) -> chrono::NaiveDate;
}

impl<FrontMatter> PostedOn for Post<FrontMatter>
where
    FrontMatter: PostedOn,
{
    fn posted_on(&self) -> chrono::NaiveDate {
        self.frontmatter.posted_on()
    }
}

pub trait ByRecency<Item> {
    fn by_recency(&self) -> Vec<&Item>;
}

impl<T> ByRecency<T> for Vec<T>
where
    T: PostedOn,
{
    fn by_recency(&self) -> Vec<&T> {
        let mut v: Vec<_> = self.iter().collect();

        v.sort_by_key(|item| std::cmp::Reverse(item.posted_on()));

        v
    }
}
