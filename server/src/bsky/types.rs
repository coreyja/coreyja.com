use rsky_lexicon::app::bsky::feed::PostView;
use rsky_lexicon::app::bsky::feed::ThreadViewPostEnum;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPostThreadData {
    pub thread: ThreadViewPostEnum,
}
