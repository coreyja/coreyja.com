use crate::*;

use sqlx::SqliteConnection;

#[async_trait]
pub trait Findable {
    async fn find(id: i64, pool: &mut SqliteConnection) -> Result<Self>
    where
        Self: Sized;

    fn id(&self) -> i64;
}

#[derive(Debug)]
pub enum QueryOnRead<T> {
    Id(i64),
    Hydrated(T),
}

impl<T> QueryOnRead<T>
where
    T: Findable,
{
    pub fn id(&self) -> i64 {
        match self {
            QueryOnRead::Id(id) => *id,
            QueryOnRead::Hydrated(val) => val.id(),
        }
    }

    #[allow(dead_code)]
    pub async fn hydrate(self, pool: &mut SqliteConnection) -> color_eyre::Result<T> {
        Ok(match self {
            QueryOnRead::Id(id) => T::find(id, pool).await?,
            QueryOnRead::Hydrated(val) => val,
        })
    }
}

impl<T> From<i64> for QueryOnRead<T> {
    fn from(val: i64) -> Self {
        QueryOnRead::Id(val)
    }
}

impl<T> From<T> for QueryOnRead<T>
where
    T: Findable,
{
    fn from(val: T) -> Self {
        QueryOnRead::Hydrated(val)
    }
}
