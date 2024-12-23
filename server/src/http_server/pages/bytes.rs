use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::NaiveDate;
use cja::{app_state::AppState as _, color_eyre::eyre::eyre};
use color_eyre::eyre::{Context as _, ContextCompat};
use itertools::Itertools;
use maud::{html, Render};

use crate::{
    http_server::{
        errors::ServerError,
        templates::{base_constrained, header::OpenGraph},
        LinkTo,
    },
    AppState,
};

#[derive(Debug, Clone)]
pub(crate) struct Byte {
    pub slug: String,
    pub subdomain: String,
    pub display_name: String,
    pub release_date: NaiveDate,
    pub short_description: String,
}

impl Byte {
    pub fn cookd_url(&self) -> String {
        format!("https://{}.cookd.dev/{}", self.subdomain, self.slug)
    }
}

pub fn get_levels() -> Vec<Byte> {
    vec![Byte {
        slug: "level-0-0".to_string(),
        subdomain: "coreyja".to_string(),
        display_name: "Level 0-0".to_string(),
        release_date: NaiveDate::from_ymd_opt(2024, 9, 3).unwrap(),
        short_description: "First ever Byte Challenge! A simple CLI Todo list to get you started. Written in Rust but don't worry its pretty language agnostic.".to_string(),
    },
    Byte {
        slug: "cdn".to_string(),
        subdomain: "coreyja".to_string(),
        display_name: "CDN".to_string(),
        release_date: NaiveDate::from_ymd_opt(2024, 9, 10).unwrap(),
        short_description: "This is a 'real life' bug! This is an actual diff from my sites repo! I was trying to integrate ImgProxy to serve different sizes and formats of my images and ran into the following bug, and thought it would make for a fun challenge! Hope you enjoy it!".to_string(),
    },
    Byte {
        slug: "websocket-chat".to_string(),
        subdomain: "coreyja".to_string(),
        display_name: "Websocket Chat".to_string(),
        release_date: NaiveDate::from_ymd_opt(2024, 9, 17).unwrap(),
        short_description: "Build a websocket chat server and client! We wanted to build a simple chat app and use websockets to sync messages across clients. But there are a few bugs to find and fix along the way!".to_string(),
    },
    Byte {
        slug: "color-blending".to_string(),
        subdomain: "coreyja".to_string(),
        display_name: "Color Blending".to_string(),
        release_date: NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
        short_description: "Build a color blending CLI! Given two colors we want to blend them together to produce a color that is a mix of the two. We've already got the start of the CLI built out for you, but there are a few bugs to find and fix along the way!".to_string(),
    }]
}

pub(crate) fn get_most_recent_bytes() -> Vec<Byte> {
    get_levels()
        .into_iter()
        .sorted_by_key(|b| b.release_date)
        .rev()
        .collect()
}

impl LinkTo for Byte {
    fn relative_link(&self) -> String {
        format!("/bytes/{}", self.slug)
    }
}

pub(crate) struct ByteList(Vec<Byte>);

impl ByteList {
    pub fn new(bytes: Vec<Byte>) -> Self {
        Self(bytes)
    }
}

impl Render for ByteList {
    fn render(&self) -> maud::Markup {
        maud::html! {
            ul {
                @for level in &self.0 {
                  li class="mb-4" {
                    a class="text-xl block underline" href=(level.relative_link()) { (level.display_name) }
                    p class="text-sm text-gray-500 mb-4 " { (level.release_date.format("%B %d, %Y").to_string()) }

                    p class="text-gray-500" { (level.short_description) }

                  }
                }
              }
        }
    }
}

fn bytes_warning() -> maud::Markup {
    maud::html! {
        div."rounded-md bg-yellow-50 p-4 my-4" {
            div."flex" {
                div."shrink-0" {
                    svg."size-5 text-yellow-400" fill="currentColor" aria-hidden="true" viewBox="0 0 20 20" data-slot="icon" {
                        path fill-rule="evenodd" clip-rule="evenodd" d="M8.485 2.495c.673-1.167 2.357-1.167 3.03 0l6.28 10.875c.673 1.167-.17 2.625-1.516 2.625H3.72c-1.347 0-2.189-1.458-1.515-2.625L8.485 2.495ZM10 5a.75.75 0 0 1 .75.75v3.5a.75.75 0 0 1-1.5 0v-3.5A.75.75 0 0 1 10 5Zm0 9a1 1 0 1 0 0-2 1 1 0 0 0 0 2Z" {}
                    }
                }
                div."ml-3" {
                    h3."text-sm font-medium text-yellow-800" {
                        "Bytes are currently disabled"
                    }
                    div."mt-2 text-sm text-yellow-700" {
                        p {
                            "Bytes are currently broken, due to losing a DB that didn't have backups. I'm working on rebuilding them, but in the meantime they will break below and have been removed from the homepage."
                        }
                    }
                }
            }
        }
    }
}

pub(crate) async fn bytes_index() -> Result<impl IntoResponse, ServerError> {
    Ok(base_constrained(
        maud::html! {
          h1 class="text-3xl mb-4" { "Bytes - Coding Challenges" }

          (bytes_warning())

          p class="mb-4" {
            "Bytes are bite-sized coding challenges that are designed to be fun and educational. "
            "They are a great way to practice your coding skills and learn new things."
          }

          p class="mb-4" {
            "New Bytes will come out ever week, so keep checking back to solve more
            puzzles and make your way up the leaderboard"
          }

          p {
            a class="text-lg" href="/bytes_leaderboard" {
                span class="text-xl" { "🥇" }
                span class="underline pl-2" { "View Overall Leaderboard" }
            }
          }

          h2 class="text-2xl mt-8 mb-4" { "Most Recent Bytes" }
          (ByteList::new(get_most_recent_bytes()))
        },
        OpenGraph::default(),
    ))
}

pub(crate) async fn byte_get(Path(slug): Path<String>) -> Result<impl IntoResponse, ServerError> {
    let cookd_levels = get_levels();
    let byte = cookd_levels.into_iter().find(|c| c.slug == slug);
    let byte =
        byte.ok_or_else(|| ServerError(eyre!("Cookd level not found"), StatusCode::NOT_FOUND))?;

    Ok(base_constrained(
        maud::html! {
          h1 class="text-3xl mb-1" { "Byte - " (byte.display_name) }
          p class="text-gray-500 mb-4" { (byte.release_date.format("%B %d, %Y").to_string()) }

          p class="mb-4" { (byte.short_description) }

          a class="text-xl mb-4 block underline" href=(format!("/bytes/{slug}/leaderboard")) { "View Leaderboard" }

          iframe class="w-full min-h-screen" src=(byte.cookd_url()) {}
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

        if username == "anonymous" {
            return None;
        }

        Some(html! {
            img."h-11 w-11 rounded-full" src=(format!("https://github.com/{username}.png")) alt=(format!("Github Avatar for {username}")) {}
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
    let cookd = get_levels();
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
            h1 class="text-3xl mb-1" { "Leaderboard for "  (cookd.display_name) }
            p class="text-gray-500 mb-4" { (cookd.release_date.format("%B %d, %Y").to_string()) }

            p class="mb-4" { (cookd.short_description) }

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
                @if rank == 1 {
                    span class="text-xl pr-3" { "🥇" }
                } @else if rank == 2 {
                    span class="text-xl pr-3" { "🥈" }
                } @else if rank == 3 {
                    span class="text-xl pr-3" { "🥉" }
                }
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

pub struct OverallLeaderboardEntry {
    pub player_github_username: Option<String>,
    pub sum: Option<i64>,
    pub count: Option<i64>,
}

pub async fn fetch_overall_leaderboard(
    app_state: &AppState,
) -> cja::Result<Vec<OverallLeaderboardEntry>> {
    let scores = sqlx::query_as!(
        OverallLeaderboardEntry,
        r#"
            SELECT player_github_username, sum(score), count(*)
            FROM CookdWebhooks
            WHERE player_github_username != 'anonymous'
            GROUP BY player_github_username
            ORDER BY sum(score) DESC
            "#
    )
    .fetch_all(app_state.db())
    .await
    .context("Could not fetch scores")?;

    Ok(scores)
}

pub(crate) async fn overall_leaderboard(
    State(app_state): State<AppState>,
) -> Result<impl IntoResponse, ServerError> {
    let scores = fetch_overall_leaderboard(&app_state).await?;

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
            h1 class="text-3xl mb-4" { "Overall Leaderboard" }

            p class="mb-4" {
                "This leaderboard shows the total score for each user across all Bytes."
                "Compete against your friends and the community to see who can solve the most Bytes and get the highest score!"
            }

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
                @if rank == 1 {
                    span class="text-xl pr-3" { "🥇" }
                } @else if rank == 2 {
                    span class="text-xl pr-3" { "🥈" }
                } @else if rank == 3 {
                    span class="text-xl pr-3" { "🥉" }
                }
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
