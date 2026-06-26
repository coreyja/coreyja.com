use std::sync::{Arc, LazyLock};

use axum::extract::State;
use chrono::{NaiveDate, Utc};
use maud::{html, Markup};
use posts::{
    blog::{BlogPosts, Track},
    notes::NotePosts,
};
use tracing::instrument;

use crate::{
    http_server::{
        errors::ServerError,
        templates::{base_constrained, header::OpenGraph},
        LinkTo,
    },
    AppState,
};

/// Year 1 accountability window. Half-open: `[WINDOW_START, WINDOW_END)`.
static WINDOW_START: LazyLock<NaiveDate> =
    LazyLock::new(|| NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
static WINDOW_END: LazyLock<NaiveDate> =
    LazyLock::new(|| NaiveDate::from_ymd_opt(2027, 6, 1).unwrap());

const BATTLESNAKE_TARGET: u32 = 48;
const OTHER_TARGET: u32 = 12;
/// Projected-finish is too noisy with only a handful of artifacts — suppress it
/// until at least this many days have elapsed (~4 weeks).
const PROJECTION_MIN_DAYS: i64 = 28;

#[derive(Debug, Clone)]
struct Artifact {
    date: NaiveDate,
    title: String,
    href: String,
    track: Track,
}

#[derive(Debug, Clone)]
struct TrackStats {
    track: Track,
    target: u32,
    count: u32,
    expected_by_now: f64,
    elapsed_fraction: f64,
    weeks_left: u32,
    need_per_week: f64,
    on_pace: bool,
    projected_finish: Option<f64>, // None until PROJECTION_MIN_DAYS elapsed
    artifacts: Vec<Artifact>,      // newest first
}

#[instrument(skip_all)]
pub(crate) async fn pace_dashboard(
    State(state): State<AppState>,
    State(blog_posts): State<Arc<BlogPosts>>,
    State(note_posts): State<Arc<NotePosts>>,
) -> Result<Markup, ServerError> {
    let today = Utc::now().date_naive();
    let artifacts = collect_artifacts(&blog_posts, &note_posts, today);

    let track_stats: Vec<TrackStats> = [
        (Track::Battlesnake, BATTLESNAKE_TARGET),
        (Track::Other, OTHER_TARGET),
    ]
    .into_iter()
    .map(|(track, target)| compute_track_stats(track, target, today, &artifacts))
    .collect();

    let og = OpenGraph::default_for_path(&state.app, "/pace");

    Ok(base_constrained(render_dashboard(&track_stats, today), og))
}

/// Half-open window membership, excluding future-dated posts.
fn in_window(date: NaiveDate, today: NaiveDate) -> bool {
    date >= *WINDOW_START && date < *WINDOW_END && date <= today
}

fn collect_artifacts(blog: &BlogPosts, notes: &NotePosts, today: NaiveDate) -> Vec<Artifact> {
    let mut artifacts: Vec<Artifact> = Vec::new();

    for post in blog.posts() {
        if in_window(post.frontmatter.date, today) {
            artifacts.push(Artifact {
                date: post.frontmatter.date,
                title: post.frontmatter.title.clone(),
                href: post.relative_link(),
                track: post.frontmatter.track,
            });
        }
    }

    for note in &notes.posts {
        if in_window(note.frontmatter.date, today) {
            artifacts.push(Artifact {
                date: note.frontmatter.date,
                title: note.frontmatter.title.clone(),
                href: note.relative_link(),
                track: note.frontmatter.track,
            });
        }
    }

    // Newest first — the list doubles as public proof-of-work.
    artifacts.sort_by_key(|a| std::cmp::Reverse(a.date));
    artifacts
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
fn compute_track_stats(
    track: Track,
    target: u32,
    today: NaiveDate,
    artifacts: &[Artifact],
) -> TrackStats {
    let elapsed_days_i64 = (today - *WINDOW_START).num_days().max(0);
    let elapsed_days = elapsed_days_i64 as f64;
    let total_days = (*WINDOW_END - *WINDOW_START).num_days() as f64;
    let elapsed_fraction = (elapsed_days / total_days).clamp(0.0, 1.0);

    let count = artifacts.iter().filter(|a| a.track == track).count() as u32;
    let expected_by_now = elapsed_fraction * f64::from(target);

    let days_remaining = (*WINDOW_END - today).num_days().max(0);
    let weeks_left = ((days_remaining as f64) / 7.0).ceil() as u32;
    let need_per_week = if weeks_left == 0 {
        0.0
    } else {
        (f64::from(target) - f64::from(count)).max(0.0) / f64::from(weeks_left)
    };

    let on_pace = f64::from(count) >= expected_by_now;

    let projected_finish = if elapsed_days_i64 >= PROJECTION_MIN_DAYS && elapsed_fraction > 0.0 {
        Some((f64::from(count) / elapsed_fraction).round())
    } else {
        None
    };

    let track_artifacts: Vec<Artifact> = artifacts
        .iter()
        .filter(|a| a.track == track)
        .cloned()
        .collect();

    TrackStats {
        track,
        target,
        count,
        expected_by_now,
        elapsed_fraction,
        weeks_left,
        need_per_week,
        on_pace,
        projected_finish,
        artifacts: track_artifacts,
    }
}

fn track_label(track: Track) -> &'static str {
    match track {
        Track::Battlesnake => "Battlesnake",
        Track::Other => "Other",
    }
}

fn render_dashboard(stats: &[TrackStats], today: NaiveDate) -> Markup {
    html! {
        h1 class="text-3xl mb-2" { "Pace" }
        p class="text-subtitle mb-8" {
            "Published artifacts toward the Year 1 target, through June 1 2027. "
            "Counted from post data — no manual entry. As of " (today) "."
        }
        @for track in stats {
            (render_track_section(track))
        }
    }
}

fn render_track_section(stats: &TrackStats) -> Markup {
    let fill_class = if stats.on_pace {
        "bg-green-500"
    } else {
        "bg-amber-500"
    };
    let pill_class = if stats.on_pace {
        "bg-green-500/20 text-green-400"
    } else {
        "bg-amber-500/20 text-amber-400"
    };
    let pill_text = if stats.on_pace {
        "On pace"
    } else {
        "Behind pace"
    };

    // Width of the filled portion of the progress bar, clamped to the bar.
    let fill_pct = if stats.target == 0 {
        0.0
    } else {
        (f64::from(stats.count) / f64::from(stats.target) * 100.0).clamp(0.0, 100.0)
    };
    let tick_pct = (stats.elapsed_fraction * 100.0).clamp(0.0, 100.0);

    html! {
        section class="mb-12" {
            div class="flex items-baseline justify-between flex-wrap gap-2" {
                h2 class="text-xl font-medium" { (track_label(stats.track)) }
                span class=(format!("px-2 py-1 rounded text-sm {pill_class}")) { (pill_text) }
            }

            div class="my-2" {
                span class="text-5xl font-medium" { (stats.count) }
                span class="text-2xl text-subtitle" { " / " (stats.target) }
            }

            // Progress bar with an expected-by-now tick marking where pace says
            // the count "should" be right now.
            div class="relative w-full h-3 bg-neutral-700 rounded my-4" {
                div class=(format!("absolute top-0 left-0 h-3 rounded {fill_class}"))
                    style=(format!("width: {fill_pct:.2}%")) {}
                div class="absolute top-[-4px] h-5 w-0.5 bg-white"
                    style=(format!("left: {tick_pct:.2}%"))
                    title="Expected by now" {}
            }

            p class="text-2xl font-medium" {
                (format!("{:.1}/wk to hit {}", stats.need_per_week, stats.target))
            }
            p class="text-subtitle" {
                (format!("{} weeks left", stats.weeks_left))
            }
            @if let Some(projected) = stats.projected_finish {
                p class="text-subtitle text-sm" {
                    (format!("Projected finish: {projected:.0}"))
                }
            }

            @if stats.artifacts.is_empty() {
                p class="text-subtitle text-sm mt-4" { "No artifacts in this track yet." }
            } @else {
                ul class="mt-4" {
                    @for artifact in &stats.artifacts {
                        li class="my-4" {
                            a href=(artifact.href) {
                                span class="text-subtitle text-sm inline-block w-[80px]" { (artifact.date) }
                                " "
                                (artifact.title)
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn artifact(track: Track, date: NaiveDate) -> Artifact {
        Artifact {
            date,
            title: "T".to_string(),
            href: "/x".to_string(),
            track,
        }
    }

    fn battlesnake_artifacts(n: usize) -> Vec<Artifact> {
        (0..n)
            .map(|_| artifact(Track::Battlesnake, *WINDOW_START))
            .collect()
    }

    #[test]
    fn in_window_includes_dates_inside_window_and_at_or_before_today() {
        let today = *WINDOW_START + Duration::days(30);
        assert!(!in_window(*WINDOW_START - Duration::days(1), today));
        assert!(!in_window(*WINDOW_END, today)); // half-open: end excluded
        assert!(in_window(*WINDOW_START, today));
        assert!(in_window(today, today));
        assert!(!in_window(today + Duration::days(1), today)); // future excluded
    }

    #[test]
    fn compute_track_stats_at_window_start() {
        let today = *WINDOW_START;
        let stats = compute_track_stats(Track::Battlesnake, 48, today, &[]);
        assert_eq!(stats.count, 0);
        assert!((stats.expected_by_now - 0.0).abs() < 1e-9);
        assert!(stats.on_pace);
        assert_eq!(stats.projected_finish, None);
        assert_eq!(stats.weeks_left, 53); // ceil(365/7) = 53
    }

    #[test]
    fn compute_track_stats_midway_on_pace() {
        let today = *WINDOW_START + Duration::days(182);
        let artifacts = battlesnake_artifacts(24);
        let stats = compute_track_stats(Track::Battlesnake, 48, today, &artifacts);
        assert!((stats.expected_by_now - 23.934_246_575_342_465).abs() < 1e-9);
        assert!(stats.on_pace);
        // weeks_left = ceil(183/7) = 27; need_per_week = 24/27
        assert!((stats.need_per_week - 24.0 / 27.0).abs() < 1e-9);
    }

    #[test]
    fn compute_track_stats_midway_behind_pace() {
        let today = *WINDOW_START + Duration::days(182);
        let artifacts = battlesnake_artifacts(10);
        let stats = compute_track_stats(Track::Battlesnake, 48, today, &artifacts);
        assert!(!stats.on_pace);
        assert!((stats.need_per_week - 38.0 / 27.0).abs() < 1e-9);
    }

    #[test]
    fn compute_track_stats_at_or_past_end() {
        let today = *WINDOW_END + Duration::days(5);
        let stats = compute_track_stats(Track::Battlesnake, 48, today, &[]);
        assert_eq!(stats.weeks_left, 0);
        assert!((stats.need_per_week - 0.0).abs() < 1e-9);
        assert!((stats.elapsed_fraction - 1.0).abs() < 1e-9);
    }

    #[test]
    fn compute_track_stats_count_exceeds_target() {
        let today = *WINDOW_START + Duration::days(60);
        let artifacts = battlesnake_artifacts(60);
        let stats = compute_track_stats(Track::Battlesnake, 48, today, &artifacts);
        assert!((stats.need_per_week - 0.0).abs() < 1e-9);
        assert!(stats.on_pace);
    }

    #[test]
    fn projected_finish_hidden_before_week_4() {
        let today = *WINDOW_START + Duration::days(10);
        let artifacts = battlesnake_artifacts(3);
        let stats = compute_track_stats(Track::Battlesnake, 48, today, &artifacts);
        assert_eq!(stats.projected_finish, None);
    }

    #[test]
    fn projected_finish_visible_after_week_4() {
        let today = *WINDOW_START + Duration::days(35);
        let artifacts = battlesnake_artifacts(5);
        let stats = compute_track_stats(Track::Battlesnake, 48, today, &artifacts);
        // (5.0 / (35.0 / 365.0)).round() = 52
        assert_eq!(stats.projected_finish, Some(52.0));
    }

    #[test]
    fn compute_track_stats_filters_only_requested_track() {
        let today = *WINDOW_START + Duration::days(60);
        let mut artifacts = Vec::new();
        for _ in 0..3 {
            artifacts.push(artifact(Track::Battlesnake, *WINDOW_START));
        }
        for _ in 0..5 {
            artifacts.push(artifact(Track::Other, *WINDOW_START));
        }
        let bs = compute_track_stats(Track::Battlesnake, 48, today, &artifacts);
        let other = compute_track_stats(Track::Other, 12, today, &artifacts);
        assert_eq!(bs.count, 3);
        assert_eq!(other.count, 5);
    }
}
