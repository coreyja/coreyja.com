use maud::{html, Markup, PreEscaped};

use crate::http_server::templates::{base_constrained, header::OpenGraph};

fn experience(job_title: &str, company: &str, timespan: &str, body: &Markup) -> Markup {
    html! {
        div class="mb-4" {
            div class="flex flex-wrap justify-between items-baseline gap-x-4" {
                h3 class="font-semibold text-base" { (job_title) }
                @if !company.is_empty() {
                    span class="text-text/70" { (company) }
                }
                span class="text-text/60 text-sm ml-auto" { (timespan) }
            }
            div class="mt-1 text-sm leading-relaxed text-text/80 space-y-1" {
                (body)
            }
        }
    }
}

fn section(title: &str, content: &Markup) -> Markup {
    html! {
        div class="mb-6" {
            h2 class="text-lg font-bold border-b border-text/20 pb-1 mb-3" { (title) }
            (content)
        }
    }
}

fn work_experience() -> Markup {
    html! {
        (experience(
            "CTO",
            "Wellsheet",
            "October 2019 – Present",
            &html! {
                p { "Lead engineering for a healthcare startup building EHR-integrated clinical tools, growing the team from early stage to a team of four engineers" }
                p { "Own infrastructure, architecture, and technical strategy across the full stack" }
                p { "Built and maintained the background job system interfacing with EHRs to ensure operations with the most up-to-date patient data" }
            },
        ))
        (experience(
            "Senior Software Engineer",
            "Betterment",
            "June 2015 – September 2019",
            &html! {
                p { "Promoted to lead the Cashflow team, focusing on the web app and developing new features" }
                p { "Helped design and build the account aggregation product, enabling users to link accounts across financial institutions" }
                p { "Architected backend systems for data storage, daily updates, and customer-facing data" }
                p { "Optimized the Two-Way Sweep algorithm that automatically transfers money between checking and savings to maintain 3–5 weeks of spending money" }
            },
        ))
        (experience(
            "Consulting Intern",
            "West Monroe Partners",
            "Summer 2014",
            &html! {
                p { "Rewrote HTML/CSS templates for a client .NET application and created a mobile version with iOS and Android native wrappers" }
            },
        ))
        (experience(
            "Freelance Web Developer",
            "",
            "August 2010 – 2014",
            &html! {
                p { "Developed websites and web apps for various clients, contracting regularly with MadLab Media Group in Longmont, CO" }
            },
        ))
    }
}

fn projects() -> Markup {
    html! {
        (experience(
            "Battlesnake",
            "battlesnake.io",
            "January 2026 – Present",
            &html! {
                p { "Took ownership of the Battlesnake competitive programming platform and community" }
            },
        ))
        (experience(
            "coreyja.com",
            "",
            "Ongoing",
            &html! {
                p { "Personal site and blog built from scratch in Rust with Axum, featuring a podcast, project showcases, and a newsletter" }
            },
        ))
    }
}

const PRINT_STYLES: &str = r#"
    @media print {
        @page { margin: 0.5in; size: letter; }
        body { font-size: 11pt !important; background: white !important; color: black !important; }
        nav, footer, header { display: none !important; }
        .resume-wrapper { max-width: none !important; padding: 0 !important; margin: 0 !important; }
        .no-print { display: none !important; }
        a { color: inherit !important; text-decoration: none !important; }
    }
"#;

pub(crate) async fn resume() -> Markup {
    base_constrained(
        html! {
            style { (PreEscaped(PRINT_STYLES)) }

            div class="resume-wrapper py-8 print:py-0" {
                div class="mb-6 text-center" {
                    h1 class="text-3xl font-bold" { "Corey Alexander" }
                    p class="text-text/70 mt-1" {
                        "coreyja.com"
                        span class="mx-2" { "·" }
                        "github.com/coreyja"
                        span class="mx-2" { "·" }
                        "contact@coreyja.com"
                    }
                }

                (section("Work Experience", &work_experience()))
                (section("Projects", &projects()))
                (section("Education", &html! {
                    (experience(
                        "Rose-Hulman Institute of Technology",
                        "CS / SE / Mathematics",
                        "Class of 2015",
                        &html! {
                            p { "Bachelor of Science — Triple Major in Computer Science, Software Engineering, and Mathematics" }
                        },
                    ))
                }))
            }
        },
        OpenGraph {
            title: "Resume — Corey Alexander".to_owned(),
            description: Some("Corey Alexander's professional resume".to_owned()),
            ..Default::default()
        },
    )
}
