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

pub trait ByRecency {
    type Item;

    fn by_recency(&self) -> Vec<&Self::Item>;
}

impl<T> ByRecency for Vec<T>
where
    T: PostedOn,
{
    type Item = T;

    fn by_recency(&self) -> Vec<&Self::Item> {
        let mut v: Vec<_> = self.iter().collect();

        v.sort_by(|a, b| b.posted_on().cmp(&a.posted_on()).reverse());

        v
    }
}
