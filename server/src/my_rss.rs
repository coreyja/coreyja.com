use crate::*;

#[derive(Debug, Clone)]
pub(crate) struct RssConfig {
    upwork_url: String,
    discord_notification_channel_id: u64,
}

impl RssConfig {
    pub(crate) fn from_env() -> Result<Self> {
        Ok(Self {
            upwork_url: std::env::var("UPWORK_RSS_URL")
                .wrap_err("Missing UPWORK_RSS_URL needed for app launch")?,
            discord_notification_channel_id: std::env::var("UPWORK_DISCORD_CHANNEL_ID")
                .wrap_err("Missing UPWORK_DISCORD_CHANNEL_ID")?
                .parse()?,
        })
    }
}

pub(crate) async fn run_rss(config: Config, discord_client: Arc<CacheAndHttp>) -> Result<()> {
    let sleep_duration = std::time::Duration::from_secs(60);

    let client = reqwest::Client::new();

    loop {
        run_upwork_rss(&config, &discord_client, &client).await?;

        tokio::time::sleep(sleep_duration).await;
    }
}

async fn run_upwork_rss(
    config: &Config,
    discord_client: &CacheAndHttp,
    client: &Client,
) -> Result<()> {
    let resp = client
        .get(&config.rss.upwork_url)
        .send()
        .await?
        .bytes()
        .await?;
    let channel = Channel::read_from(&resp[..])?;

    for item in channel.items() {
        process_upwork_job_rss(&config, item, discord_client).await?;
    }

    Ok(())
}

async fn process_upwork_job_rss(
    config: &Config,
    item: &rss::Item,
    discord_client: &CacheAndHttp,
) -> Result<()> {
    let guid = &item.guid().unwrap().value;

    let existing_record_id = sqlx::query!("SELECT id FROM UpworkJobs where guid = ?", guid)
        .fetch_optional(&config.db_pool)
        .await?
        .map(|r| r.id);

    if let Some(_) = existing_record_id {
        info!(guid, "We already recorded this job");
    } else {
        let title = item.title().unwrap();
        let content = item.content().unwrap();

        let new_record_id = sqlx::query!(
            "INSERT INTO UpworkJobs (guid, title, content) VALUES (?,?,?) RETURNING id",
            guid,
            title,
            content
        )
        .fetch_one(&config.db_pool)
        .await?
        .id;

        info!(guid, upwork_job_id = new_record_id, "Added new UpworkJob");

        fn truncate(input: &str, max_chars: usize) -> &str {
            match input.char_indices().nth(max_chars) {
                None => input,
                Some((idx, _)) => &input[..idx],
            }
        }

        ChannelId(config.rss.discord_notification_channel_id)
            .send_message(&discord_client.http, |m| {
                m.add_embed(|e| {
                    e.title(title)
                        .description(truncate(content, 200))
                        .url(guid)
                        .color(Color::from_rgb(17, 138, 0))
                })
            })
            .await?;

        info!(
            upwork_job_id = new_record_id,
            "Sent Discord message for Job"
        );
    }

    Ok(())
}
