use axum::{
    extract::{Form, Path, State},
    response::{IntoResponse, Redirect},
};
use maud::html;
use serde::Deserialize;
use sqlx::query;
use uuid::Uuid;

use crate::{http_server::admin::Timestamp, state::AppState};
use db::tool_suggestions::ToolSuggestion;

use super::super::{
    auth::session::AdminUser,
    errors::ServerError,
    templates::{base_constrained, header::OpenGraph},
};

#[allow(clippy::too_many_lines)]
pub(crate) async fn tool_suggestions_list(
    _admin: AdminUser,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let suggestions = ToolSuggestion::list_pending(&app_state.db).await?;

    // Get thread IDs for each suggestion's stitch
    let mut suggestions_with_thread_info = Vec::new();
    for suggestion in suggestions {
        let thread_info = query!(
            r#"
            SELECT thread_id FROM stitches WHERE stitch_id = $1
        "#,
            suggestion.previous_stitch_id
        )
        .fetch_optional(&app_state.db)
        .await?;

        suggestions_with_thread_info.push((suggestion, thread_info));
    }

    Ok(base_constrained(
        html! {
            h1 class="text-xl mb-4" { "Tool Suggestions" }

            p class="mb-4 text-gray-600" {
                "Agents can suggest new tools they'd like to have. Review pending suggestions below."
            }

            @if suggestions_with_thread_info.is_empty() {
                div class="bg-gray-100 p-4 rounded" {
                    p { "No pending tool suggestions at the moment." }
                }
            } @else {
                div class="space-y-4" {
                    @for (suggestion, thread_info) in suggestions_with_thread_info {
                        div class="border rounded-lg p-4 bg-white shadow-sm" {
                            div class="mb-3" {
                                h3 class="text-lg font-semibold" { (suggestion.name) }
                                p class="text-gray-700 mt-1" { (suggestion.description) }
                            }

                            div class="mb-4" {
                                h4 class="font-medium mb-2" { "Examples:" }
                                @for (idx, example) in suggestion.examples.as_array().unwrap_or(&vec![]).iter().enumerate() {
                                    div class="bg-gray-50 p-3 rounded mb-3" {
                                        span class="text-sm text-gray-600 font-medium" { "Example " (idx + 1) }

                                        div class="grid grid-cols-1 md:grid-cols-2 gap-3 mt-2" {
                                            div {
                                                span class="text-xs text-gray-500 block mb-1" { "Input:" }
                                                pre class="text-xs overflow-x-auto bg-white p-2 rounded border" {
                                                    (serde_json::to_string_pretty(example.get("input").unwrap_or(&serde_json::json!(null))).unwrap_or_default())
                                                }
                                            }

                                            div {
                                                span class="text-xs text-gray-500 block mb-1" { "Output:" }
                                                pre class="text-xs overflow-x-auto bg-white p-2 rounded border" {
                                                    (serde_json::to_string_pretty(example.get("output").unwrap_or(&serde_json::json!(null))).unwrap_or_default())
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            div class="flex gap-2 items-end" {
                                form action={"/admin/tool-suggestions/" (suggestion.suggestion_id) "/dismiss"} method="post" class="flex gap-2 items-end" {
                                    input type="text" name="linear_ticket_id"
                                        placeholder="Linear Ticket ID (e.g., ENG-123)"
                                        class="px-3 py-1 border rounded"
                                        required="required";
                                    button type="submit"
                                        class="px-4 py-1 bg-blue-500 text-white rounded hover:bg-blue-600" {
                                        "Dismiss with Ticket"
                                    }
                                }

                                form action={"/admin/tool-suggestions/" (suggestion.suggestion_id) "/skip"} method="post" {
                                    button type="submit"
                                        class="px-4 py-1 bg-gray-500 text-white rounded hover:bg-gray-600" {
                                        "Skip"
                                    }
                                }
                            }

                            div class="flex justify-between items-center text-xs text-gray-500 mt-2" {
                                div {
                                    "Suggested: " (Timestamp(suggestion.created_at))
                                }
                                @if let Some(thread_info) = thread_info {
                                    a href={"/admin/threads?thread=" (thread_info.thread_id) "&stitch=" (suggestion.previous_stitch_id)}
                                       target="_blank"
                                       class="text-blue-500 hover:text-blue-700 underline" {
                                        "View in Thread Graph →"
                                    }
                                }
                            }
                        }
                    }
                }
            }

        },
        OpenGraph::default(),
    ))
}

#[derive(Deserialize)]
pub(crate) struct DismissRequest {
    linear_ticket_id: String,
}

pub(crate) async fn dismiss_suggestion(
    _admin: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Form(payload): Form<DismissRequest>,
) -> Result<impl IntoResponse, ServerError> {
    let suggestion = ToolSuggestion::get_by_id(&app_state.db, id)
        .await?
        .ok_or_else(|| color_eyre::eyre::eyre!("Tool suggestion not found"))?;

    if suggestion.status != "pending" {
        return Err(color_eyre::eyre::eyre!("Tool suggestion is not pending").into());
    }

    ToolSuggestion::dismiss(&app_state.db, id, payload.linear_ticket_id).await?;

    Ok(Redirect::to("/admin/tool-suggestions"))
}

pub(crate) async fn skip_suggestion(
    _admin: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, ServerError> {
    let suggestion = ToolSuggestion::get_by_id(&app_state.db, id)
        .await?
        .ok_or_else(|| color_eyre::eyre::eyre!("Tool suggestion not found"))?;

    if suggestion.status != "pending" {
        return Err(color_eyre::eyre::eyre!("Tool suggestion is not pending").into());
    }

    ToolSuggestion::skip(&app_state.db, id).await?;

    Ok(Redirect::to("/admin/tool-suggestions"))
}
