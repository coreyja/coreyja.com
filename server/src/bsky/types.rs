use rsky_lexicon::app::bsky::feed::ThreadViewPostEnum;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPostThreadData {
    pub thread: ThreadViewPostEnum,
}
