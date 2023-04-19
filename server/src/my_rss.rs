use tracing::instrument;

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
                .into_diagnostic()
                .wrap_err("Missing UPWORK_RSS_URL needed for app launch")?,
            discord_notification_channel_id: std::env::var("UPWORK_DISCORD_CHANNEL_ID")
                .into_diagnostic()
                .wrap_err("Missing UPWORK_DISCORD_CHANNEL_ID")?
                .parse()
                .into_diagnostic()?,
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

#[instrument(skip_all)]
async fn run_upwork_rss(
    config: &Config,
    discord_client: &CacheAndHttp,
    client: &Client,
) -> Result<()> {
    let resp = client
        .get(&config.rss.upwork_url)
        .send()
        .await
        .into_diagnostic()?
        .bytes()
        .await
        .into_diagnostic()?;
    let channel = Channel::read_from(&resp[..]).into_diagnostic()?;

    for item in channel.items() {
        process_upwork_job_rss(config, item, discord_client).await?;
    }

    Ok(())
}

#[instrument(skip_all)]
async fn process_upwork_job_rss(
    config: &Config,
    item: &rss::Item,
    discord_client: &CacheAndHttp,
) -> Result<()> {
    let guid = &item.guid().unwrap().value;

    let existing_record_id = sqlx::query!("SELECT id FROM UpworkJobs where guid = ?", guid)
        .fetch_optional(&config.db_pool)
        .await
        .into_diagnostic()?
        .map(|r| r.id);

    if existing_record_id.is_some() {
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
        .await
        .into_diagnostic()?
        .id;

        info!(guid, upwork_job_id = new_record_id, "Added new UpworkJob");

        fn truncate(input: &str, max_chars: usize) -> &str {
            match input.char_indices().nth(max_chars) {
                None => input,
                Some((idx, _)) => &input[..idx],
            }
        }

        let proposal_url = config
            .app
            .app_url(&format!("/admin/upwork/proposals/{new_record_id}"));

        ChannelId(config.rss.discord_notification_channel_id)
            .send_message(&discord_client.http, |m| {
                m.add_embed(|e| {
                    e.title(title)
                        .description(truncate(content, 200))
                        .url(guid)
                        .color(Color::from_rgb(17, 138, 0))
                })
                .add_embed(|e| {
                    e.title("Create Proposal Here")
                        .url(proposal_url)
                        .color(Color::from_rgb(17, 138, 0))
                })
            })
            .await
            .into_diagnostic()?;

        info!(
            upwork_job_id = new_record_id,
            "Sent Discord message for Job"
        );
    }

    Ok(())
}
