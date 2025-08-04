use axum::{
    extract::State,
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Form, Router,
};
use maud::html;
use serde::Deserialize;

use crate::{
    memory::blocks::{MemoryBlock, MemoryBlockType},
    state::AppState,
};

use super::super::{
    auth::session::AdminUser,
    errors::ServerError,
    templates::{base_constrained, header::OpenGraph},
};

pub(crate) fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(view_persona))
        .route("/edit", get(edit_persona_form).post(update_persona))
}

async fn view_persona(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let persona = MemoryBlock::get_persona(&state.db).await?;

    let content = html! {
        div class="space-y-6" {
            div class="flex items-center justify-between" {
                h2 class="text-2xl font-bold text-gray-900" { "Persona Configuration" }
                a href="/admin/persona/edit" class="px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600" {
                    "Edit Persona"
                }
            }

            div class="bg-white shadow rounded-lg p-6" {
                @if let Some(persona) = persona {
                    div class="space-y-4" {
                        div {
                            h3 class="text-lg font-medium text-gray-900" { "Current Persona" }
                            p class="mt-1 text-sm text-gray-500" {
                                "Last updated: " (persona.updated_at.format("%Y-%m-%d %H:%M:%S UTC"))
                            }
                        }
                        div class="mt-4" {
                            pre class="whitespace-pre-wrap bg-gray-50 rounded-lg p-4 text-sm" {
                                (persona.content)
                            }
                        }
                    }
                } @else {
                    div class="text-center py-12" {
                        p class="text-gray-500" { "No persona has been configured yet." }
                        a href="/admin/persona/edit" class="mt-4 inline-block px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600" {
                            "Create Persona"
                        }
                    }
                }
            }
        }
    };

    Ok(base_constrained(html! { (content) }, OpenGraph::default()))
}

async fn edit_persona_form(
    _admin: AdminUser,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let persona = MemoryBlock::get_persona(&state.db).await?;

    let content = html! {
        div class="space-y-6" {
            div class="flex items-center justify-between" {
                h2 class="text-2xl font-bold text-gray-900" { "Edit Persona" }
                a href="/admin/persona" class="text-blue-500 hover:text-blue-700 hover:underline" {
                    "← Back to Persona"
                }
            }

            div class="bg-white shadow rounded-lg p-6" {
                form action="/admin/persona/edit" method="post" class="space-y-6" {
                    div {
                        label for="content" class="block text-sm font-medium text-gray-700" {
                            "Persona Content"
                        }
                        p class="mt-1 text-sm text-gray-500" {
                            "Define the AI assistant's persona, behavior, and instructions. This will be included in the system prompt."
                        }
                        textarea
                            id="content"
                            name="content"
                            rows="20"
                            class="mt-2 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                            placeholder="You are a helpful AI assistant..."
                            required {
                            @if let Some(persona) = &persona {
                                (persona.content)
                            }
                        }
                    }

                    div class="flex justify-end space-x-3" {
                        a href="/admin/persona" class="px-4 py-2 bg-gray-500 text-white rounded hover:bg-gray-600" {
                            "Cancel"
                        }
                        button
                            type="submit"
                            class="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600" {
                            @if persona.is_some() { "Update Persona" } @else { "Create Persona" }
                        }
                    }
                }
            }
        }
    };

    Ok(base_constrained(html! { (content) }, OpenGraph::default()))
}

#[derive(Deserialize)]
struct UpdatePersonaForm {
    content: String,
}

async fn update_persona(
    _admin: AdminUser,
    State(state): State<AppState>,
    Form(form): Form<UpdatePersonaForm>,
) -> Result<Response, ServerError> {
    let existing_persona = MemoryBlock::get_persona(&state.db).await?;

    if let Some(persona) = existing_persona {
        MemoryBlock::update_content(&state.db, persona.memory_block_id, form.content).await?;
    } else {
        MemoryBlock::create(&state.db, MemoryBlockType::Persona, form.content).await?;
    }

    Ok(Redirect::to("/admin/persona").into_response())
}
