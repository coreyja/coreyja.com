use super::*;

pub(crate) fn make_router(syntax_css: String) -> Router<AppState> {
    Router::new()
        .route("/static/*path", get(static_assets))
        .route("/styles/syntax.css", get(|| async move { syntax_css }))
        .route("/styles/tailwind.css", get(|| async { TAILWIND_STYLES }))
        .route("/", get(pages::home::home_page))
        .route("/twitch_oauth", get(api::external::twitch_oauth::handler))
        .route("/github_oauth", get(api::external::github_oauth::handler))
        .route(
            "/admin/upwork/proposals/:id",
            get(pages::admin::upwork_proposal_get),
        )
        .route(
            "/admin/upwork/proposals/:id",
            post(pages::admin::upwork_proposal_post),
        )
        .route("/posts/rss.xml", get(pages::blog::rss_feed))
        .route(
            "/rss.xml",
            get(|| async { Redirect::permanent("/posts/rss.xml") }),
        )
        .route("/posts", get(pages::blog::posts_index))
        .route(
            "/posts/weekly/",
            // I accidently published by first newsletter under this path
            // so I'm redirecting it to the newsletter home page. I'll
            // update the few links I made outside this blog to the correct link
            get(|| async { Redirect::permanent("/newsletter") }),
        )
        .route("/posts/*key", get(pages::blog::post_get))
        .route("/til", get(pages::til::til_index))
        .route("/til/:slug", get(pages::til::til_get))
        .route("/tags/*tag", get(redirect_to_posts_index))
        .route("/year/*year", get(redirect_to_posts_index))
        .route("/newsletter", get(newsletter_get))
        .fallback(fallback)
}
