use regex::Regex;
use rsky_lexicon::app::bsky::feed::{ThreadViewPost, ThreadViewPostEnum};
use types::GetPostThreadData;
use url::Url;

pub mod types;

pub async fn fetch_thread(post_url: &str) -> cja::Result<ThreadViewPost> {
    let re = Regex::new(r"/profile/([\w.:]+)/post/([\w]+)").unwrap();
    let caps = re.captures(post_url).unwrap();

    let did = caps.get(1).unwrap().as_str();
    let post_id = caps.get(2).unwrap().as_str();

    let at_proto_uri = format!("at://{did}/app.bsky.feed.post/{post_id}");
    let mut url = Url::parse("https://public.api.bsky.app/xrpc/app.bsky.feed.getPostThread")?;
    url.set_query(Some(&format!("uri={at_proto_uri}")));

    let res = reqwest::get(url).await?;
    let data = res.json::<GetPostThreadData>().await?;

    let ThreadViewPostEnum::ThreadViewPost(thread) = data.thread else {
        return Err(cja::color_eyre::eyre::eyre!("Expected thread view post"));
    };

    Ok(thread)
}
