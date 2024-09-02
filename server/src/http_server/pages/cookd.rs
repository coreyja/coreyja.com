use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use cja::{app_state::AppState as _, color_eyre::eyre::eyre};
use color_eyre::eyre::{Context as _, ContextCompat};
use maud::html;

use crate::{
    http_server::{
        errors::ServerError,
        templates::{base_constrained, header::OpenGraph},
        LinkTo,
    },
    AppState,
};

struct CookdLevel {
    slug: String,
    subdomain: String,
    display_name: String,
}

impl CookdLevel {
    fn cookd_url(&self) -> String {
        format!("https://{}.cookd.dev/{}", self.subdomain, self.slug)
    }
}

fn get_cookd_levels() -> Vec<CookdLevel> {
    vec![
        CookdLevel {
            slug: "level-0-0".to_string(),
            subdomain: "coreyja".to_string(),
            display_name: "Level 0-0".to_string(),
        },
        CookdLevel {
            slug: "Level-1-1".to_string(),
            subdomain: "corey".to_string(),
            display_name: "Level 1-1".to_string(),
        },
    ]
}

impl LinkTo for CookdLevel {
    fn relative_link(&self) -> String {
        format!("/cookd_demo/{}", self.slug)
    }
}

pub(crate) async fn cookd_index(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(base_constrained(
        maud::html! {
          h1 { "Cookd Demo Index" }

          p { "This is a demo of the Cookd platform." }

          p {
            a href="/cookd_leaderboard" { "View Overall Leaderboard" }
          }

          ul {
            @for cookd in get_cookd_levels() {
              li {
                a href=(cookd.relative_link()) { (cookd.slug) }
              }
            }
          }
        },
        OpenGraph::default(),
    ))
}

pub(crate) async fn cookd_get(
    Path(slug): Path<String>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let cookd = get_cookd_levels();
    let cookd = cookd.into_iter().find(|c| c.slug == slug);
    let cookd =
        cookd.ok_or_else(|| ServerError(eyre!("Cookd level not found"), StatusCode::NOT_FOUND))?;

    Ok(base_constrained(
        maud::html! {
          h1 { "Cookd - "  (cookd.display_name) }

          a href=(format!("/cookd_demo/{slug}/leaderboard")) { "View Leadboard" }

          iframe class="w-full min-h-screen" src=(cookd.cookd_url()) {}
        },
        OpenGraph::default(),
    ))
}

struct ScoreEntry {
    player_github_username: Option<String>,
    score: i64,
}

impl ScoreEntry {
    fn display_username(&self) -> &str {
        self.player_github_username
            .as_deref()
            .unwrap_or("Anonymous")
    }

    fn avatar(&self) -> Option<maud::Markup> {
        let username = self.player_github_username.as_ref()?;

        Some(html! {
            img."h-11 w-11 rounded-full" src=(format!("https://github.com/{}.png", username)) alt="" {}
        })
    }

    fn avatar_render(&self) -> maud::Markup {
        self.avatar().unwrap_or_else(|| html! {})
    }
}

pub(crate) async fn single_leaderboard(
    Path(slug): Path<String>,
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let cookd = get_cookd_levels();
    let cookd = cookd.into_iter().find(|c| c.slug == slug);
    let cookd =
        cookd.ok_or_else(|| ServerError(eyre!("Cookd level not found"), StatusCode::NOT_FOUND))?;

    let scores = sqlx::query_as!(
        ScoreEntry,
        r#"
            SELECT player_github_username, score
            FROM CookdWebhooks
            WHERE slug = $1 and subdomain = $2
            ORDER BY score DESC
            "#,
        slug,
        cookd.subdomain
    )
    .fetch_all(app_state.db())
    .await
    .context("Could not fetch scores")?;

    Ok(base_constrained(
        maud::html! {
          h1 { "Leaderboard - "  (cookd.slug) }

        div."px-4 sm:px-6 lg:px-8" {
            div."mt-8 flow-root" {
                div."-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8" {
                    div."inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8" {
                        table."min-w-full divide-y divide-gray-300 table-fixed" {
                            thead {
                                tr {
                                    th."py-3.5 text-left text-sm font-semibold text-gray-900 w-0 pr-4" scope="col" {
                                        "Rank"
                                    }
                                    th."py-3.5 px-2 w-11" scope="col" {
                                        // Spacer row for Avatar but no heading label
                                    }
                                    th."py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900" scope="col" {
                                        "Github Username"
                                    }
                                    th."px-3 py-3.5 text-left text-sm font-semibold text-gray-900" scope="col" {
                                        "Score"
                                    }
                                }
                            }

                            tbody."divide-y divide-gray-200" {
                                @for (i, score) in scores.into_iter().enumerate() {
                                    (single_leaderboard_row(i+1, score))
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

fn single_leaderboard_row(rank: usize, entry: ScoreEntry) -> maud::Markup {
    maud::html! {
        tr {
            td."whitespace-nowrap py-5 pr-4 text-xl font-extrabold text-gray-900" {
                (rank)
            }
            td."whitespace-nowrap px-2" {
                div."ml-4 h-11 w-11 flex-shrink-0" {
                    ( entry.avatar_render() )
                }
            }
            td."whitespace-nowrap py-5 pl-4 pr-3 text-sm sm:pl-0" {
                div."flex items-center" {
                    div."ml-4" {
                        div."font-medium text-gray-900" {
                            @if let Some(username) = entry.player_github_username {
                                a."underline" href=(format!("https://github.com/{username}")) target="_blank" rel="noopener noreferrer" {
                                    (username)
                                }
                            } @else {
                                "Anonymous"
                            }
                        }
                    }
                }
            }
            td."whitespace-nowrap px-3 py-5 text-sm text-gray-500" {
                div."text-gray-900" {
                    (entry.score)
                }
            }
        }
    }
}

struct OverallScoreEntry {
    inner: ScoreEntry,
    count: i64,
}

pub(crate) async fn overall_leaderboard(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let scores = sqlx::query!(
        r#"
            SELECT player_github_username, sum(score), count(*)
            FROM CookdWebhooks
            GROUP BY player_github_username
            ORDER BY sum(score) DESC
            "#
    )
    .fetch_all(app_state.db())
    .await
    .context("Could not fetch scores")?;

    let scores = scores
        .into_iter()
        .map(|row| {
            Ok(OverallScoreEntry {
                inner: ScoreEntry {
                    player_github_username: row.player_github_username,
                    score: row.sum.context("No summed score found")?,
                },
                count: row.count.context("No count of entries found")?,
            })
        })
        .collect::<color_eyre::Result<Vec<_>>>()?;

    Ok(base_constrained(
        maud::html! {
          h1 { "Overall Leaderboard" }

        div."px-4 sm:px-6 lg:px-8" {
            div."mt-8 flow-root" {
                div."-mx-4 -my-2 overflow-x-auto sm:-mx-6 lg:-mx-8" {
                    div."inline-block min-w-full py-2 align-middle sm:px-6 lg:px-8" {
                        table."min-w-full divide-y divide-gray-300 table-fixed" {
                            thead {
                                tr {
                                    th."py-3.5 text-left text-sm font-semibold text-gray-900 w-0 pr-4" scope="col" {
                                        "Rank"
                                    }
                                    th."py-3.5 px-2 w-11" scope="col" {
                                        // Spacer row for Avatar but no heading label
                                    }
                                    th."py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900" scope="col" {
                                        "Github Username"
                                    }
                                    th."px-3 py-3.5 text-left text-sm font-semibold text-gray-900" scope="col" {
                                        "Total Score"
                                    }
                                    th."px-3 py-3.5 text-left text-sm font-semibold text-gray-900" scope="col" {
                                        "Bytes Submitted"
                                    }
                                }
                            }

                            tbody."divide-y divide-gray-200" {
                                @for (i, score) in scores.into_iter().enumerate() {
                                    (overall_leaderboard_row(i+1, score))
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

fn overall_leaderboard_row(rank: usize, entry: OverallScoreEntry) -> maud::Markup {
    maud::html! {
        tr {
            td."whitespace-nowrap py-5 pr-4 text-xl font-extrabold text-gray-900" {
                (rank)
            }
            td."whitespace-nowrap px-2" {
                div."ml-4 h-11 w-11 flex-shrink-0" {
                    ( entry.inner.avatar_render() )
                }
            }
            td."whitespace-nowrap py-5 pl-4 pr-3 text-sm sm:pl-0" {
                div."flex items-center" {
                    div."ml-4" {
                        div."font-medium text-gray-900" {
                            @if let Some(username) = entry.inner.player_github_username {
                                a."underline" href=(format!("https://github.com/{username}")) target="_blank" rel="noopener noreferrer" {
                                    (username)
                                }
                            } @else {
                                "Anonymous"
                            }
                        }
                    }
                }
            }
            td."whitespace-nowrap px-3 py-5 text-sm text-gray-500" {
                div."text-gray-900" {
                    (entry.inner.score)
                }
            }
            td."whitespace-nowrap px-3 py-5 text-sm text-gray-500" {
                div."text-gray-900" {
                    (entry.count)
                }
            }
        }
    }
}
