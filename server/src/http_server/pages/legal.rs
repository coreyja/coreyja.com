use maud::{html, Markup};

use crate::http_server::{
    errors::ServerError,
    templates::{base_constrained, header::OpenGraph},
};

#[allow(clippy::too_many_lines)]
pub(crate) async fn privacy_policy() -> Result<Markup, ServerError> {
    Ok(base_constrained(
        html! {
          div class="max-w-prose" {
            h1 class="my-4 text-2xl" {
                "Privacy Policy"
            }

            p class="my-4" {
              "I want to be as transparent as possible about the data I collect and how it is used."
            }

            h3 class="my-4 text-xl" {
              "Analytics"
            }

            p class="my-4" {
              "I do not run any Javascript, either first or third party, on this site. However, I do use "
              "server side analytics to get a basic idea of which posts and pages are most popular."
              br;
              "These server side analytics do NOT include the IP address of visitors."
            }

            p class="my-4" {
              "If you are logged in, the server side analytics will include your internal user ID."
            }

            p class="my-4" {
              "I'm using "
              a href="https://posthog.com" target="_blank" class="underline" { "Posthog" }
              " to store and analyze analytics data."
            }

            h3 class="my-4 text-xl" {
              "Newsletter"
            }

            p class="my-4" {
              "I have a newsletter that you can subscribe to. I use "
              a href="https://buttondown.com/" target="_blank"  class="underline" { "Buttondown" }
              " to manage this list and send emails. "
              "You can sign up for my newsletter at the bottom of each page on this site. "
              "This form submits directly to Buttondown who will store your email address."
            }

            h3 class="my-4 text-xl" {
              "Embedded Content"
            }

            p class="my-4" {
              "While I do not run any Javascript on this site, I do embed content from other sites which may. Each embedded "
              "service is described below."
            }

            h4 class="my-2 text-lg" {
              "Youtube"
            }

            p class="my-4" {
              "I embed my Youtube videos on this site. Youtube may collect data about you, use cookies, "
              "embed additional third-party tracking, and monitor your interaction with that embedded content. "
              "You can view their privacy policy at "
              a href="https://policies.google.com/privacy" target="_blank" class="underline" { "https://policies.google.com/privacy" }
              "."
            }

            h4 class="my-2 text-lg" {
              "Cookd"
            }

            p class="my-4" {
              "I embed content from Cookd.dev to power the Interactive Byte Challenges. This is done by "
              "adding an iframe that loads content from cookd.dev."
              "You can view their privacy policy at "
              a href="https://cookd.dev/privacy-policy" target="_blank" rel="noopener noreferrer" class="underline" { "https://cookd.dev/privacy-policy" }
              "."
            }

            h3 class="my-4 text-xl" {
              "Hosting"
            }

            p class="my-4" {
              "This site, as well as other services it relies, are is hosted on "
              a href="https://fly.io" target="_blank" class="underline" { "fly.io" }
              "."
            }

            p class="mt-4" {
              "As of Aug 12 2024 these additional services include self hosted instances of the following:"
            }

            ul class="mb-4" {
              li {
                a href="https://imgproxy.dev" target="_blank" class="underline" { "imgproxy" }
              }
            }

            p class="my-4" {
              "Some static assets are served directly from Amazon S3."
            }

            h3 class="my-4 text-xl" {
              "Authentication"
            }

            p class="my-4" {
              "This site has a login system, which uses Github OAuth."
            }

            p class="my-4" {
              "I only store the Github user ID and username of users who have logged in. "
            }

            p class="my-4" {
              "There is also an administrator panel that uses Google Oauth (to interface with the Youtube API), "
              "however this is NOT exposed to the public."
            }

            h3 class="my-4 text-xl" {
              "Other Linked Third Party Services"
            }

            p class="my-4" {
              "I am using Google Fonts to load the \"Quicksand\" font for this site. "
              "This loads the font using a CSS link tag to avoid running Google Javascript."
            }

            p class="my-4" {
              "I am using Font Awesome for icons. This is also loaded using a CSS link tag."
            }

            h3 class="mt-8 my-4 text-xl" {
              "Anything Else"
            }

            p class="my-4" {
              "Email me at "
              a href="mailto:privacy@coreyja.com" class="underline" { "privacy@coreyja.com" }
              " if you have any questions!"
            }
          }

        },
        OpenGraph::default(),
    ))
}
