use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use crate::AppState;

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Event {
    api_key: String,
    #[allow(clippy::struct_field_names)]
    event: String,
    properties: serde_json::Map<String, serde_json::Value>,
}

pub(crate) async fn capture_event(
    state: &AppState,
    event_name: &str,
    user_id: Option<&Uuid>,
    properties: Option<serde_json::Map<String, serde_json::Value>>,
) -> color_eyre::Result<()> {
    let Some(posthog_key) = state.posthog_key.clone() else {
        debug!("No Posthog key configured, skipping event capture");

        return Ok(());
    };

    let mut properties = properties.unwrap_or_default();

    if let Some(d) = properties.get("distinct_id") {
        tracing::info!(
            event_name = event_name,
            "distinct_id is already set to {:?} and will be overridden",
            d
        );
    }

    if let Some(user_id) = user_id {
        properties.insert(
            "distinct_id".to_string(),
            serde_json::Value::String(user_id.to_string()),
        );
    } else {
        properties.insert(
            "distinct_id".to_string(),
            serde_json::Value::String(Uuid::new_v4().to_string()),
        );
        properties.insert(
            "$process_person_profile".to_string(),
            serde_json::Value::Bool(false),
        );
    };

    let event = Event {
        api_key: posthog_key,
        event: event_name.to_string(),
        properties,
    };

    reqwest::Client::new()
        .post("https://us.i.posthog.com/capture/")
        .json(&event)
        .send()
        .await?;

    Ok(())
}
