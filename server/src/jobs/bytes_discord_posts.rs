use chrono::{DateTime, Utc};
use cja::jobs::Job;
use color_eyre::eyre::{Context as _, ContextCompat as _};
use db::DiscordChannel;
use serde::{Deserialize, Serialize};
use serenity::all::ChannelId;
use uuid::Uuid;

use crate::{
    http_server::pages::bytes::{fetch_overall_leaderboard, get_levels},
    AppState,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostByteSubmission {
    pub cookd_webhook_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CookdWebhook {
    #[allow(clippy::struct_field_names)]
    cookd_webhook_id: Uuid,
    player_github_username: Option<String>,
    player_github_email: Option<String>,
    score: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    slug: String,
    subdomain: String,
}

#[async_trait::async_trait]
impl Job<AppState> for PostByteSubmission {
    const NAME: &'static str = "PostByteSubmission";

    async fn run(&self, app_state: AppState) -> cja::Result<()> {
        let record = sqlx::query_as!(
            CookdWebhook,
            "SELECT * FROM CookdWebhooks WHERE cookd_webhook_id = $1",
            self.cookd_webhook_id
        )
        .fetch_one(&app_state.db)
        .await
        .context("Could not find webhook record")?;

        let levels = get_levels();
        let level = levels
            .iter()
            .find(|level| level.slug == record.slug)
            .wrap_err("Level not found")?;

        let level_leaderboard_entries = sqlx::query_as!(
            CookdWebhook,
            "SELECT * FROM CookdWebhooks WHERE subdomain = $1 AND slug = $2 ORDER BY score DESC",
            record.subdomain,
            record.slug
        )
        .fetch_all(&app_state.db)
        .await?;

        let level_leaderboard_position = level_leaderboard_entries
            .iter()
            .position(|entry| entry.cookd_webhook_id == record.cookd_webhook_id)
            .wrap_err("Could not find position in leaderboard")?
            + 1;

        let overall_leaderboard_entries = fetch_overall_leaderboard(&app_state)
            .await
            .context("Could not fetch overall leaderboard")?;

        let overall_leaderboard_position = overall_leaderboard_entries
            .iter()
            .position(|entry| entry.player_github_username == record.player_github_username)
            .map(|pos| pos + 1);

        let level_msg = format!(
            "{} just scored {} points on {}. Nicely done!\n\nThis puts them in {} place out of {} players for this Byte!",
            record
                .player_github_username
                .unwrap_or_else(|| "Anonymous Player".to_string()),
            record.score,
            level.display_name,
            level_leaderboard_position,
            level_leaderboard_entries.len()
        );

        let overall_msg = if let Some(overall_leaderboard_position) = overall_leaderboard_position {
            format!(
                "{level_msg}\nAnd puts them in {} place out of {} players on the over all leaderboard!",
                overall_leaderboard_position,
                overall_leaderboard_entries.len()
            )
        } else {
            level_msg
        };

        let channels = sqlx::query_as!(
            DiscordChannel,
            "SELECT * FROM DiscordChannels WHERE purpose = 'byte_submissions'"
        )
        .fetch_all(&app_state.db)
        .await?;

        let Some(ref discord) = app_state.discord else {
            tracing::info!("Discord not configured, skipping Discord post");
            return Ok(());
        };

        for channel in channels {
            let create_message = serenity::all::CreateMessage::new().content(&overall_msg);
            let channel_id = ChannelId::new(channel.channel_id.parse::<u64>()?);

            channel_id.send_message(discord, create_message).await?;
        }

        Ok(())
    }
}
