use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use maud::{html, Markup};
use serde::{Deserialize, Serialize};

use crate::{
    http_server::templates::{base_constrained, header::OpenGraph},
    pexels::{fetch_user_photos, Photo},
    AppState,
};

fn render_grid(
    photos: &[Photo],
    current_page: i32,
    has_next_page: bool,
    total_results: i32,
) -> Markup {
    base_constrained(
        html! {
            body class="bg-gray-100 min-h-screen" {
                div class="container mx-auto px-4 py-8" {
                    // Header with pagination info
                    div class="mb-8 text-center" {
                        h1 class="text-3xl font-bold text-gray-800 mb-2" {
                            "Pexels Photo Gallery"
                        }
                        p class="text-gray-600" {
                            "Page " (current_page) " • Showing "
                            (((current_page - 1) * 20) + 1) " to "
                            (std::cmp::min(current_page * 20, total_results))
                            " of " (total_results) " photos"
                        }
                    }

                    // Photo grid container
                    div class="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-6 mb-8" {
                        @for photo in photos {
                            div class="relative group overflow-hidden rounded-lg shadow-lg bg-white" {
                                a href=(photo.url) target="_blank" class="block" {
                                    div class="aspect-w-1 aspect-h-1 w-full" {
                                        img
                                            src=(photo.src.medium)
                                            alt=(format!("Photo by {}", photo.photographer))
                                            class="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
                                            loading="lazy"
                                        ;
                                    }

                                    div class="absolute inset-0 bg-black bg-opacity-50 opacity-0 group-hover:opacity-100 transition-opacity duration-300 flex items-end" {
                                        div class="w-full p-4 text-white" {
                                            p class="text-sm font-medium" {
                                                "By " (photo.photographer)
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Pagination navigation
                    div class="flex justify-center gap-4 mt-8" {
                        @if current_page > 1 {
                            a
                                href=(format!("/?page={}", current_page - 1))
                                class="bg-white hover:bg-gray-50 text-gray-800 font-semibold py-2 px-6 rounded-lg border transition-colors duration-300"
                            {
                                "Previous Page"
                            }
                        }

                        @if has_next_page {
                            a
                                href=(format!("/?page={}", current_page + 1))
                                class="bg-blue-500 hover:bg-blue-600 text-white font-semibold py-2 px-6 rounded-lg transition-colors duration-300"
                            {
                                "Next Page"
                            }
                        }
                    }
                }
            }
        },
        OpenGraph::default(),
    )
}

#[derive(Serialize, Deserialize)]
pub struct PageParams {
    page: Option<i32>,
}

pub async fn pexels_index(
    Query(params): Query<PageParams>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, crate::http_server::errors::ServerError> {
    let current_page = params.page.unwrap_or(1);

    let response = fetch_user_photos("coreyja", &state.pexels.api_key, current_page).await?;
    let has_next_page = response.next_page.is_some();
    let total_results = response.total_results;
    let markup = render_grid(&response.photos, current_page, has_next_page, total_results);
    Ok(markup)
}
