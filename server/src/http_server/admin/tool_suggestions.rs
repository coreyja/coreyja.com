use axum::{extract::State, response::IntoResponse};
use maud::html;

use crate::state::AppState;
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

    Ok(base_constrained(
        html! {
            h1 class="text-xl mb-4" { "Tool Suggestions" }

            p class="mb-4 text-gray-600" {
                "Agents can suggest new tools they'd like to have. Review pending suggestions below."
            }

            @if suggestions.is_empty() {
                div class="bg-gray-100 p-4 rounded" {
                    p { "No pending tool suggestions at the moment." }
                }
            } @else {
                div class="space-y-4" {
                    @for suggestion in suggestions {
                        div class="border rounded-lg p-4 bg-white shadow-sm" {
                            div class="mb-3" {
                                h3 class="text-lg font-semibold" { (suggestion.name) }
                                p class="text-gray-700 mt-1" { (suggestion.description) }
                            }

                            div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4" {
                                div {
                                    h4 class="font-medium mb-2" { "Sample Inputs:" }
                                    @for (idx, input) in suggestion.sample_inputs.as_array().unwrap_or(&vec![]).iter().enumerate() {
                                        div class="bg-gray-50 p-2 rounded mb-2" {
                                            span class="text-sm text-gray-500" { "Example " (idx + 1) ":" }
                                            pre class="text-xs overflow-x-auto" { (serde_json::to_string_pretty(input).unwrap_or_default()) }
                                        }
                                    }
                                }

                                div {
                                    h4 class="font-medium mb-2" { "Sample Outputs:" }
                                    @for (idx, output) in suggestion.sample_outputs.as_array().unwrap_or(&vec![]).iter().enumerate() {
                                        div class="bg-gray-50 p-2 rounded mb-2" {
                                            span class="text-sm text-gray-500" { "Example " (idx + 1) ":" }
                                            pre class="text-xs overflow-x-auto" { (serde_json::to_string_pretty(output).unwrap_or_default()) }
                                        }
                                    }
                                }
                            }

                            div class="flex gap-2 items-end" {
                                div class="flex gap-2 items-end" {
                                    input type="text" id={"linear-ticket-" (suggestion.suggestion_id)}
                                        placeholder="Linear Ticket ID (e.g., ENG-123)"
                                        class="px-3 py-1 border rounded";
                                    button onclick={"dismissSuggestion('" (suggestion.suggestion_id) "')"}
                                        class="px-4 py-1 bg-blue-500 text-white rounded hover:bg-blue-600" {
                                        "Dismiss with Ticket"
                                    }
                                }

                                button onclick={"skipSuggestion('" (suggestion.suggestion_id) "')"}
                                    class="px-4 py-1 bg-gray-500 text-white rounded hover:bg-gray-600" {
                                    "Skip"
                                }
                            }

                            div class="text-xs text-gray-500 mt-2" {
                                "Suggested: " (format_timestamp(&suggestion.created_at))
                            }
                        }
                    }
                }
            }

            script {
                (maud::PreEscaped(r"
                async function dismissSuggestion(id) {
                    const ticketInput = document.getElementById('linear-ticket-' + id);
                    const ticketId = ticketInput.value.trim();
                    
                    if (!ticketId) {
                        alert('Please enter a Linear ticket ID');
                        return;
                    }
                    
                    try {
                        const response = await fetch(`/admin/api/tool-suggestions/${id}/dismiss`, {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json',
                            },
                            body: JSON.stringify({ linear_ticket_id: ticketId })
                        });
                        
                        if (response.ok) {
                            window.location.reload();
                        } else {
                            alert('Failed to dismiss suggestion');
                        }
                    } catch (error) {
                        alert('Error: ' + error.message);
                    }
                }
                
                async function skipSuggestion(id) {
                    try {
                        const response = await fetch(`/admin/api/tool-suggestions/${id}/skip`, {
                            method: 'POST',
                            headers: {
                                'Content-Type': 'application/json',
                            }
                        });
                        
                        if (response.ok) {
                            window.location.reload();
                        } else {
                            alert('Failed to skip suggestion');
                        }
                    } catch (error) {
                        alert('Error: ' + error.message);
                    }
                }
                "))
            }
        },
        OpenGraph::default(),
    ))
}

fn format_timestamp(timestamp: &chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = now - *timestamp;
    chrono_humanize::HumanTime::from(duration).to_string()
}
