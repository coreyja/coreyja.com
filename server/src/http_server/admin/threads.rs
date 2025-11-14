use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
};
use cja::app_state::AppState as _;
use color_eyre::eyre::Context;
use db::agentic_threads::{Stitch, StitchType, Thread, ThreadStatus, ThreadType};
use db::discord_threads::DiscordThreadMetadata;
use maud::{html, Markup, PreEscaped};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    al::standup::{Content, Message},
    state::AppState,
};

use super::super::{
    auth::session::AdminUser,
    errors::ServerError,
    templates::{base_constrained, header::OpenGraph},
};
use super::Timestamp;

#[derive(Deserialize)]
pub(crate) struct ThreadListQuery {
    #[serde(default)]
    days: Option<i32>,
}

/// Main thread list page
pub(crate) async fn threads_list(
    _admin: AdminUser,
    State(app_state): State<AppState>,
    Query(query): Query<ThreadListQuery>,
) -> Result<impl IntoResponse, ServerError> {
    // Default to 3 days if not specified, cap at 7 days maximum
    let days = query.days.unwrap_or(3).clamp(1, 7);

    let threads = Thread::list_within_days(app_state.db(), days)
        .await
        .context("Failed to fetch threads")?;

    // Collect threads with counts
    let mut threads_with_counts = Vec::new();
    for thread in threads {
        let stitch_count = thread
            .count_stitches(app_state.db())
            .await
            .context("Failed to count stitches")?;

        let children_count = thread
            .count_children(app_state.db())
            .await
            .context("Failed to count children")?;

        threads_with_counts.push(ThreadWithCounts {
            thread,
            stitch_count,
            children_count,
        });
    }

    Ok(base_constrained(
        thread_list_page(&threads_with_counts, days),
        OpenGraph {
            title: "Admin - Threads".to_string(),
            ..Default::default()
        },
    ))
}

/// Thread detail page showing stitches, children, and parents
pub(crate) async fn thread_detail(
    _admin: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<ThreadListQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let days = query.days.unwrap_or(3).clamp(1, 7);

    let thread = Thread::get_by_id(app_state.db(), id)
        .await
        .context("Failed to fetch thread")?
        .ok_or_else(|| color_eyre::eyre::eyre!("Thread not found"))?;

    let stitches = thread
        .get_stitches(app_state.db())
        .await
        .context("Failed to fetch stitches")?;

    // Fetch Discord metadata if this is an interactive thread
    let discord_metadata = if thread.thread_type == ThreadType::Interactive {
        DiscordThreadMetadata::find_by_thread_id(app_state.db(), thread.thread_id)
            .await
            .context("Failed to fetch Discord metadata")?
    } else {
        None
    };

    // Fetch children
    let children = thread
        .get_children(app_state.db())
        .await
        .context("Failed to fetch children")?;

    let mut children_with_counts = Vec::new();
    for child in children {
        let stitch_count = child
            .count_stitches(app_state.db())
            .await
            .context("Failed to count stitches")?;

        let children_count = child
            .count_children(app_state.db())
            .await
            .context("Failed to count children")?;

        children_with_counts.push(ThreadWithCounts {
            thread: child,
            stitch_count,
            children_count,
        });
    }

    // Fetch parents
    let parents = thread
        .get_parent_chain(app_state.db())
        .await
        .context("Failed to fetch parent chain")?;

    Ok(base_constrained(
        thread_detail_page(
            &thread,
            &stitches,
            discord_metadata,
            &children_with_counts,
            &parents,
            days,
        ),
        OpenGraph {
            title: format!("Thread: {}", thread.goal),
            ..Default::default()
        },
    ))
}

/// Thread messages page showing reconstructed conversation
pub(crate) async fn thread_messages(
    _admin: AdminUser,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<ThreadListQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let days = query.days.unwrap_or(3).clamp(1, 7);

    let thread = Thread::get_by_id(app_state.db(), id)
        .await
        .context("Failed to fetch thread")?
        .ok_or_else(|| color_eyre::eyre::eyre!("Thread not found"))?;

    let messages = crate::jobs::thread_processor::reconstruct_messages(app_state.db(), id)
        .await
        .context("Failed to reconstruct messages")?;

    Ok(base_constrained(
        thread_messages_page(&thread, &messages, days),
        OpenGraph {
            title: format!("Thread Messages: {}", thread.goal),
            ..Default::default()
        },
    ))
}

// ============================================================================
// Helper structs
// ============================================================================

struct ThreadWithCounts {
    thread: Thread,
    stitch_count: i64,
    children_count: i64,
}

// ============================================================================
// Template rendering functions
// ============================================================================

fn thread_list_page(threads: &[ThreadWithCounts], days: i32) -> Markup {
    html! {
        div class="py-4" {
            div class="flex justify-between items-center mb-6" {
                h1 class="text-2xl font-bold" { "Agentic Threads" }

                form method="get" class="flex items-center gap-2" {
                    label for="days" class="text-sm" { "Show last:" }
                    select
                        name="days"
                        id="days"
                        class="border rounded px-2 py-1" {
                        option value="1" selected[days == 1] { "1 day" }
                        option value="3" selected[days == 3] { "3 days" }
                        option value="7" selected[days == 7] { "7 days" }
                    }
                    button type="submit" class="px-3 py-1 bg-blue-500 text-white rounded hover:bg-blue-600" { "Go" }
                }
            }

            @if threads.is_empty() {
                p class="text-gray-500 italic" { "No threads found in the last " (days) " day(s)." }
            } @else {
                div class="space-y-4" {
                    @for thread_with_counts in threads {
                        (thread_list_item(thread_with_counts, days))
                    }
                }
            }
        }
    }
}

fn thread_list_item(twc: &ThreadWithCounts, days: i32) -> Markup {
    let thread = &twc.thread;
    let status_color = status_color(&thread.status);

    html! {
        div class="border rounded-lg p-4 hover:shadow-md transition-shadow" {
            div class="flex items-start gap-3" {
                // Status indicator
                div
                    class="w-3 h-3 rounded-full mt-1 flex-shrink-0"
                    style=(format!("background-color: {}", status_color))
                    title=(format!("{:?}", thread.status)) {}

                div class="flex-1 min-w-0" {
                    // Thread goal and link
                    a
                        href=(format!("/admin/threads/{}?days={}", thread.thread_id, days))
                        class="text-blue-600 hover:underline font-medium block mb-2" {
                        (thread.goal)
                    }

                    // Metadata
                    div class="flex flex-wrap gap-4 text-sm text-gray-600" {
                        span { "Type: " (format!("{:?}", thread.thread_type)) }
                        span { "Stitches: " (twc.stitch_count) }
                        span { "Children: " (twc.children_count) }
                        span { "Created: " (Timestamp(thread.created_at)) }
                    }

                    // Tasks if any
                    @if let Some(tasks_array) = thread.tasks.as_array() {
                        @if !tasks_array.is_empty() {
                            div class="mt-2" {
                                (render_task_list_json(&thread.tasks))
                            }
                        }
                    }
                }
            }
        }
    }
}

fn thread_detail_page(
    thread: &Thread,
    stitches: &[Stitch],
    discord_metadata: Option<DiscordThreadMetadata>,
    children: &[ThreadWithCounts],
    parents: &[Thread],
    days: i32,
) -> Markup {
    html! {
        div class="py-4" {
            (render_thread_nav(thread.thread_id, days, true))
            (render_thread_header(thread))
            (render_parent_threads(parents, days))
            (render_thread_tasks(&thread.tasks))
            (render_thread_result(thread.result.as_ref()))
            (render_discord_metadata(discord_metadata))
            (render_stitches_section(stitches))
            (render_children_section(children, days))
        }
    }
}

fn render_thread_nav(thread_id: uuid::Uuid, days: i32, is_details: bool) -> Markup {
    html! {
        div class="mb-4" {
            a href=(format!("/admin/threads?days={}", days)) class="text-blue-600 hover:underline" {
                "← Back to thread list"
            }
        }
        div class="border-b mb-4" {
            div class="flex gap-4" {
                a
                    href=(format!("/admin/threads/{}?days={}", thread_id, days))
                    class=(if is_details { "px-4 py-2 border-b-2 border-blue-500 font-medium" } else { "px-4 py-2 text-gray-600 hover:text-gray-900" }) {
                    "Details"
                }
                a
                    href=(format!("/admin/threads/{}/messages?days={}", thread_id, days))
                    class=(if !is_details { "px-4 py-2 border-b-2 border-blue-500 font-medium" } else { "px-4 py-2 text-gray-600 hover:text-gray-900" }) {
                    "Messages"
                }
            }
        }
    }
}

fn render_thread_header(thread: &Thread) -> Markup {
    html! {
        div class="mb-6" {
            h1 class="text-2xl font-bold mb-2" { (thread.goal) }
            div class="flex items-center gap-2 mb-2" {
                span
                    class="px-2 py-1 rounded text-sm text-white"
                    style=(format!("background-color: {}", status_color(&thread.status))) {
                    (format!("{:?}", thread.status))
                }
                span class="text-gray-600 text-sm" {
                    (format!("{:?}", thread.thread_type))
                }
            }
            div class="text-sm text-gray-600 space-y-1" {
                p { "Thread ID: " code class="bg-gray-100 px-1 rounded" { (thread.thread_id) } }
                p { "Created: " (Timestamp(thread.created_at)) }
                p { "Updated: " (Timestamp(thread.updated_at)) }
            }
        }
    }
}

fn render_parent_threads(parents: &[Thread], days: i32) -> Markup {
    if parents.is_empty() {
        return html! {};
    }
    html! {
        details class="mb-6 border rounded p-4" {
            summary class="cursor-pointer font-medium" { "Parent Threads (" (parents.len()) ")" }
            div class="mt-2 space-y-2 pl-4" {
                @for parent in parents {
                    div {
                        a
                            href=(format!("/admin/threads/{}?days={}", parent.thread_id, days))
                            class="text-blue-600 hover:underline" {
                            (parent.goal)
                        }
                        span class="text-sm text-gray-600 ml-2" {
                            (format!("{:?}", parent.status))
                        }
                    }
                }
            }
        }
    }
}

fn render_thread_tasks(tasks: &serde_json::Value) -> Markup {
    if let Some(tasks_array) = tasks.as_array() {
        if !tasks_array.is_empty() {
            return html! {
                div class="mb-6" {
                    h2 class="text-xl font-bold mb-2" { "Tasks" }
                    (render_task_list_json(tasks))
                }
            };
        }
    }
    html! {}
}

fn render_thread_result(result: Option<&serde_json::Value>) -> Markup {
    let Some(result) = result else {
        return html! {};
    };
    let Some(result_obj) = result.as_object() else {
        return html! {};
    };
    let success = result_obj
        .get("success")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    html! {
        div class="mb-6" {
            h2 class="text-xl font-bold mb-2" { "Result" }
            div class=(format!("border rounded p-3 {}", if success { "border-green-500" } else { "border-red-500" })) {
                div class="font-medium mb-2" {
                    @if success {
                        span class="text-green-700" { "✓ Success" }
                    } @else {
                        span class="text-red-700" { "✗ Failed" }
                    }
                }
                @if let Some(error) = result_obj.get("error").and_then(|e| e.as_str()) {
                    pre class="text-sm bg-gray-50 p-2 rounded overflow-x-auto" { (error) }
                }
            }
        }
    }
}

fn render_discord_metadata(metadata: Option<DiscordThreadMetadata>) -> Markup {
    let Some(metadata) = metadata else {
        return html! {};
    };
    html! {
        details class="mb-6 border rounded p-4" {
            summary class="cursor-pointer font-medium" { "Discord Metadata" }
            div class="mt-2 space-y-1 text-sm" {
                p { "Thread Name: " strong { (metadata.thread_name) } }
                p { "Discord Thread ID: " code class="bg-gray-100 px-1 rounded" { (metadata.discord_thread_id) } }
                p { "Channel ID: " code class="bg-gray-100 px-1 rounded" { (metadata.channel_id) } }
                p { "Created By: " (metadata.created_by) }
                @if let Some(participants) = metadata.participants.as_array() {
                    @if !participants.is_empty() {
                        p {
                            "Participants: "
                            @for (i, participant) in participants.iter().enumerate() {
                                @if let Some(p_str) = participant.as_str() {
                                    (p_str)
                                    @if i < participants.len() - 1 {
                                        ", "
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_stitches_section(stitches: &[Stitch]) -> Markup {
    html! {
        div class="mb-6" {
            h2 class="text-xl font-bold mb-2" { "Stitches (" (stitches.len()) ")" }
            div class="space-y-3" {
                @for stitch in stitches {
                    (render_stitch(stitch))
                }
            }
        }
    }
}

fn render_children_section(children: &[ThreadWithCounts], days: i32) -> Markup {
    if children.is_empty() {
        return html! {};
    }
    html! {
        div class="mb-6" {
            h2 class="text-xl font-bold mb-2" { "Child Threads (" (children.len()) ")" }
            div class="space-y-2" {
                @for child in children {
                    div class="border rounded p-3" {
                        a
                            href=(format!("/admin/threads/{}?days={}", child.thread.thread_id, days))
                            class="text-blue-600 hover:underline font-medium" {
                            (child.thread.goal)
                        }
                        div class="text-sm text-gray-600 mt-1" {
                            span { (format!("{:?}", child.thread.status)) " · " }
                            span { (child.stitch_count) " stitches · " }
                            span { (child.children_count) " children" }
                        }
                    }
                }
            }
        }
    }
}

fn thread_messages_page(thread: &Thread, messages: &[Message], days: i32) -> Markup {
    html! {
        div class="py-4" {
            (render_thread_nav(thread.thread_id, days, false))

            // Thread header
            div class="mb-6" {
                h1 class="text-2xl font-bold mb-2" { (thread.goal) }
                div class="text-sm text-gray-600" {
                    "Thread ID: " code class="bg-gray-100 px-1 rounded" { (thread.thread_id) }
                }
            }

            // Messages
            div class="space-y-4" {
                (render_messages(messages))
            }
        }
    }
}

// ============================================================================
// Helper rendering functions
// ============================================================================

fn render_task_list_json(tasks_json: &serde_json::Value) -> Markup {
    let Some(tasks_array) = tasks_json.as_array() else {
        return html! { p class="text-red-500" { "Invalid tasks data" } };
    };

    html! {
        div class="space-y-1" {
            @for task in tasks_array {
                @if let (Some(status), Some(description)) = (task.get("status").and_then(|s| s.as_str()), task.get("description").and_then(|d| d.as_str())) {
                    div class="flex items-start gap-2 text-sm" {
                        span class="mt-0.5" { (task_status_icon_str(status)) }
                        span class=(if status == "completed" { "line-through text-gray-500" } else { "" }) {
                            (description)
                        }
                    }
                }
            }
        }
    }
}

fn task_status_icon_str(status: &str) -> &'static str {
    match status {
        "pending" => "⭕",
        "in_progress" => "🔄",
        "completed" => "✅",
        _ => "❓",
    }
}

fn status_color(status: &ThreadStatus) -> &'static str {
    match status {
        ThreadStatus::Pending => "#9CA3AF",   // gray
        ThreadStatus::Running => "#F59E0B",   // yellow/orange
        ThreadStatus::Waiting => "#3B82F6",   // blue
        ThreadStatus::Completed => "#10B981", // green
        ThreadStatus::Failed => "#EF4444",    // red
        ThreadStatus::Aborted => "#DC2626",   // dark red
    }
}

fn render_stitch(stitch: &Stitch) -> Markup {
    let stitch_icon = match stitch.stitch_type {
        StitchType::SystemPrompt => "📋",
        StitchType::InitialPrompt | StitchType::DiscordMessage => "💬",
        StitchType::LlmCall => "🤖",
        StitchType::ToolCall => "🔧",
        StitchType::ThreadResult => "📊",
        StitchType::AgentThought => "💭",
        StitchType::ClarificationRequest => "❓",
        StitchType::Error => "❌",
    };

    html! {
        details class="border rounded" {
            summary class="cursor-pointer p-3 hover:bg-gray-50" {
                span class="mr-2" { (stitch_icon) }
                strong { (format!("{:?}", stitch.stitch_type)) }
                @if let Some(tool_name) = &stitch.tool_name {
                    span class="ml-2 text-sm text-gray-600" { "(" (tool_name) ")" }
                }
                span class="ml-2 text-sm text-gray-500" {
                    (Timestamp(stitch.created_at))
                }
            }

            div class="p-3 space-y-2 bg-gray-50" {
                p class="text-xs text-gray-500" {
                    "Stitch ID: " code class="bg-white px-1 rounded" { (stitch.stitch_id) }
                }

                @if let Some(request) = &stitch.llm_request {
                    details class="mt-2" {
                        summary class="cursor-pointer text-sm font-medium" { "LLM Request" }
                        pre class="mt-2 text-xs bg-white p-2 rounded overflow-x-auto" {
                            (format!("{:#}", request))
                        }
                    }
                }

                @if let Some(response) = &stitch.llm_response {
                    details class="mt-2" {
                        summary class="cursor-pointer text-sm font-medium" { "LLM Response" }
                        pre class="mt-2 text-xs bg-white p-2 rounded overflow-x-auto" {
                            (format!("{:#}", response))
                        }
                    }
                }

                @if let Some(tool_input) = &stitch.tool_input {
                    details class="mt-2" {
                        summary class="cursor-pointer text-sm font-medium" { "Tool Input" }
                        pre class="mt-2 text-xs bg-white p-2 rounded overflow-x-auto" {
                            (format!("{:#}", tool_input))
                        }
                    }
                }

                @if let Some(tool_output) = &stitch.tool_output {
                    details class="mt-2" {
                        summary class="cursor-pointer text-sm font-medium" { "Tool Output" }
                        pre class="mt-2 text-xs bg-white p-2 rounded overflow-x-auto" {
                            (format!("{:#}", tool_output))
                        }
                    }
                }

                @if let Some(summary) = &stitch.thread_result_summary {
                    div class="mt-2" {
                        p class="text-sm font-medium" { "Result Summary:" }
                        p class="text-sm mt-1" { (summary) }
                    }
                }

                @if let Some(child_id) = &stitch.child_thread_id {
                    div class="mt-2" {
                        p class="text-sm" {
                            "Spawned child thread: "
                            a
                                href=(format!("/admin/threads/{}", child_id))
                                class="text-blue-600 hover:underline" {
                                (child_id)
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_messages(messages: &[Message]) -> Markup {
    html! {
        @for message in messages {
            (render_message(message))
        }
    }
}

fn render_message(message: &Message) -> Markup {
    let role_color = match message.role.as_str() {
        "user" => "bg-blue-50 border-blue-200",
        "assistant" => "bg-green-50 border-green-200",
        _ => "bg-gray-50 border-gray-200",
    };

    html! {
        div class=(format!("border rounded p-4 {}", role_color)) {
            div class="font-medium mb-2 text-sm" {
                (message.role)
            }

            div class="space-y-2" {
                @for content_block in &message.content {
                    (render_content_block(content_block))
                }
            }
        }
    }
}

fn render_content_block(content: &Content) -> Markup {
    match content {
        Content::Text(text_content) => {
            html! {
                div class="prose prose-sm max-w-none" {
                    (PreEscaped(text_content.text.replace('\n', "<br>")))
                }
            }
        }
        Content::ToolUse(tool_use) => {
            html! {
                details class="border rounded p-2 bg-yellow-50" {
                    summary class="cursor-pointer text-sm font-medium" {
                        "🔧 Tool Use: " (tool_use.name)
                    }
                    div class="mt-2 space-y-1 text-xs" {
                        p { "ID: " code class="bg-white px-1 rounded" { (tool_use.id) } }
                        details {
                            summary class="cursor-pointer" { "Input" }
                            pre class="bg-white p-2 rounded overflow-x-auto mt-1" {
                                (format!("{:#}", tool_use.input))
                            }
                        }
                    }
                }
            }
        }
        Content::ToolResult(tool_result) => {
            html! {
                details class="border rounded p-2 bg-purple-50" {
                    summary class="cursor-pointer text-sm font-medium" {
                        "📋 Tool Result"
                    }
                    div class="mt-2 space-y-1 text-xs" {
                        p { "Tool Use ID: " code class="bg-white px-1 rounded" { (tool_result.tool_use_id) } }
                        pre class="bg-white p-2 rounded overflow-x-auto" {
                            (tool_result.content)
                        }
                    }
                }
            }
        }
        Content::Image(image_content) => {
            html! {
                div class="border rounded p-2 bg-blue-50" {
                    div class="text-sm font-medium mb-2" {
                        "🖼️ Image"
                    }
                    @if let Some(data) = &image_content.source.data {
                        @if let Some(media_type) = &image_content.source.media_type {
                            img class="max-w-full h-auto rounded" src=(format!("data:{};base64,{}", media_type, data));
                        } @else {
                            p class="text-xs text-gray-600" { "Image (base64 encoded)" }
                        }
                    } @else if let Some(url) = &image_content.source.url {
                        img class="max-w-full h-auto rounded" src=(url);
                    }
                }
            }
        }
        Content::Document(document_content) => {
            html! {
                details class="border rounded p-2 bg-green-50" {
                    summary class="cursor-pointer text-sm font-medium" {
                        "📄 Document"
                        @if let Some(media_type) = &document_content.source.media_type {
                            span class="text-xs font-normal ml-2 text-gray-600" { "(" (media_type) ")" }
                        }
                    }
                    div class="mt-2 space-y-2" {
                        @if let Some(data) = &document_content.source.data {
                            div class="text-xs text-gray-600" {
                                "Size: ~" (data.len() * 3 / 4) " bytes"
                            }
                            @if let Some(media_type) = &document_content.source.media_type {
                                div class="space-y-2" {
                                    a
                                        href=(format!("data:{};base64,{}", media_type, data))
                                        download="document.pdf"
                                        class="inline-block px-3 py-1 bg-blue-500 text-white text-xs rounded hover:bg-blue-600"
                                    {
                                        "⬇ Download"
                                    }
                                    @if media_type == "application/pdf" {
                                        div class="border rounded overflow-hidden" {
                                            iframe
                                                src=(format!("data:{};base64,{}", media_type, data))
                                                class="w-full"
                                                style="height: 600px;"
                                                title="PDF Preview"
                                            {}
                                        }
                                    }
                                }
                            }
                        }
                        @if let Some(url) = &document_content.source.url {
                            a
                                href=(url)
                                target="_blank"
                                class="inline-block px-3 py-1 bg-blue-500 text-white text-xs rounded hover:bg-blue-600"
                            {
                                "🔗 View document"
                            }
                        }
                    }
                }
            }
        }
    }
}
