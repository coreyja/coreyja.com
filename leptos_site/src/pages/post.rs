use std::{path::PathBuf, str::FromStr};

use leptos::*;
use leptos_query::QueryResult;
use leptos_router::*;
use markdown::mdast::{self, *};
use posts::{blog::BlogPost, MarkdownAst};

#[derive(Clone)]
struct PostRendingContext {
    slug: String,
}

#[component]
pub fn MarkdownNodes(nodes: Vec<Node>) -> impl IntoView {
    nodes
        .into_iter()
        .map(|n| view! { <MarkdownNode node=n/> })
        .collect_view()
}

#[component]
fn MdImage(img: mdast::Image) -> impl IntoView {
    let context = use_context::<PostRendingContext>().unwrap();

    let mut path = PathBuf::from_str(&img.url).unwrap();

    // If the path is relative change it to be "/assets/{context.slug}"
    if path.is_relative() {
        path = PathBuf::from_str("/assets")
            .unwrap()
            .join(context.slug)
            .join(path)
    }

    view! { <img src=path.to_str().map(|x| x.to_string()) attr:title=img.title attr:alt=img.alt/> }
}

#[component]
pub fn ListInner(nodes: Vec<Node>) -> impl IntoView {
    nodes
        .into_iter()
        .map(|n| {
            view! {
                <li>
                    <MarkdownNode node=n/>
                </li>
            }
        })
        .collect_view()
}

#[component]
pub fn MarkdownNode(node: Node) -> impl IntoView {
    fn blank() -> View {
        view! {}.into_view()
    }

    match node {
        Node::Root(r) => view! { <MarkdownNodes nodes=r.children/> }.into_view(),
        Node::BlockQuote(BlockQuote {
            children,
            position: _,
        }) => view! {
            <blockquote>
                <MarkdownNodes nodes=children/>
            </blockquote>
        }
        .into_view(),
        Node::FootnoteDefinition(_) => todo!(),
        Node::MdxJsxFlowElement(_) => todo!(),
        Node::List(List {
            children, ordered, ..
        }) => {
            if ordered {
                view! {
                    <ol class="max-w-prose">
                        <ListInner nodes=children/>
                    </ol>
                }
                .into_view()
            } else {
                view! {
                    <ul class="max-w-prose">
                        <ListInner nodes=children/>
                    </ul>
                }
                .into_view()
            }
        }
        Node::MdxjsEsm(_) => todo!(),
        Node::Toml(_) => todo!(),
        Node::Yaml(_) => blank(),
        Node::Break(Break { .. }) => view! { <br/> }.into_view(),
        Node::InlineCode(InlineCode { value, .. }) => view! { <code>{value}</code> }.into_view(),
        Node::InlineMath(_) => todo!(),
        Node::Delete(Delete { children, .. }) => view! {
            <del>
                <MarkdownNodes nodes=children/>
            </del>
        }
        .into_view(),
        Node::Emphasis(Emphasis { children, .. }) => view! {
            <em>

                <MarkdownNodes nodes=children/>
            </em>
        }
        .into_view(),
        Node::MdxTextExpression(_) => todo!(),
        Node::FootnoteReference(_) => todo!(),
        Node::Html(_) => todo!(),
        Node::Image(image) => view! { <MdImage img=image/> }.into_view(),
        Node::ImageReference(_) => todo!(),
        Node::MdxJsxTextElement(_) => todo!(),
        Node::Link(Link {
            children,
            url,
            title,
            ..
        }) => view! {
            <A href=url attr:title=title>
                <MarkdownNodes nodes=children/>
            </A>
        }
        .into_view(),
        Node::LinkReference(_) => todo!(),
        Node::Strong(Strong { children, .. }) => view! {
            <strong>
                <MarkdownNodes nodes=children/>
            </strong>
        }
        .into_view(),
        Node::Text(Text { value, .. }) => value.into_view(),
        Node::Code(_) => todo!(),
        Node::Math(_) => todo!(),
        Node::MdxFlowExpression(_) => todo!(),
        Node::Heading(Heading {
            children, depth, ..
        }) => {
            let id: Option<String> = None;

            match depth {
                1 => view! {
                    <h1 class="max-w-prose text-2xl" id=id>
                        <MarkdownNodes nodes=children/>
                    </h1>
                }
                .into_view(),
                2 => view! {
                    <h2 class="max-w-prose text-xl">
                        <MarkdownNodes nodes=children/>
                    </h2>
                }
                .into_view(),
                3 => view! {
                    <h3 class="max-w-prose text-lg">
                        <MarkdownNodes nodes=children/>
                    </h3>
                }
                .into_view(),
                4 => view! {
                    <h4 class="max-w-prose text-lg text-subtitle">
                        <MarkdownNodes nodes=children/>
                    </h4>
                }
                .into_view(),
                5 => view! {
                    <h5 class="max-w-prose text-lg text-subtitle font-light">
                        <MarkdownNodes nodes=children/>
                    </h5>
                }
                .into_view(),
                6 => view! {
                    <h6 class="max-w-prose text-base text-subtitle">
                        <MarkdownNodes nodes=children/>
                    </h6>
                }
                .into_view(),
                _ => unreachable!("There aren't this many headings in HTML"),
            }
        }
        Node::Table(_) => todo!(),
        Node::ThematicBreak(_) => view! { <hr class="my-8 opacity-20"/> }.into_view(),
        Node::TableRow(_) => todo!(),
        Node::TableCell(_) => todo!(),
        Node::ListItem(_) => todo!(),
        Node::Definition(_) => todo!(),
        Node::Paragraph(Paragraph { children, .. }) => view! {
            <p class="my-4 max-w-prose leading-loose">
                <MarkdownNodes nodes=children/>
            </p>
        }
        .into_view(),
    }
}

#[component]
pub fn Markdown(ast: MarkdownAst) -> impl IntoView {
    view! { <MarkdownNode node=Node::Root(ast.0)/> }
}

#[component]
pub fn Post(post: BlogPost) -> impl IntoView {
    let mut slug = post.path.clone();
    slug.pop();
    provide_context(PostRendingContext {
        slug: format!("posts/{}", slug.to_str().unwrap()),
    });

    view! {
        <h1 class="text-2xl">{post.title().to_string()}</h1>
        <span class="block text-lg text-subtitle mb-8">{format!("{}", post.frontmatter.date)}</span>

        <div>
            <Markdown ast=post.ast()/>
        </div>
    }
}

#[server]
async fn get_post(slug: Option<String>) -> Result<Option<BlogPost>, ServerFnError> {
    let Some(slug) = slug else {
        return Ok(None);
    };
    let state = crate::server::extractors::extract_state()?;
    let post = state
        .blog_posts
        .posts()
        .iter()
        .find_map(|p| p.matches_path(&slug).map(|m| (p, m)));

    let Some(post) = post else {
        return Ok(None);
    };

    Ok(Some(post.0.clone()))
}

#[component]
pub fn PostPage() -> impl IntoView {
    let params = use_params_map();
    let slug = move || params.with(|p| p.get("slug").cloned());

    let QueryResult { data: post, .. } =
        leptos_query::use_query(slug, get_post, Default::default());

    view! {
        <Suspense>

            {move || {
                match post() {
                    Some(Ok(Some(post))) => {
                        view! { <Post post=post/> }
                    }
                    Some(Ok(None)) => view! { <div>"Post not found"</div> }.into_view(),
                    Some(Err(e)) => {
                        view! { <div>"Error loading post:" {format!("{}", e)}</div> }.into_view()
                    }
                    None => view! { <div>"Loading"</div> }.into_view(),
                }
            }}

        </Suspense>
    }
}
