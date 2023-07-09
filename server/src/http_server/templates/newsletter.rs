use maud::{html, Markup};

use super::base_constrained;

pub(crate) fn newsletter_page() -> Markup {
    base_constrained(html! {
      div  class="max-w-prose"  {
        h1 class="text-3xl mb-8" { "coreyja weekly" }

        h3 class="text-2xl text-subtitle mb-4" { "New Posts in your inbox every Friday" }
        h3 class="text-xl text-subtitle mb-12" {
          "Added to the blog on Saturday"
          br;
          "Sign up below to read one day early!"
        }

        p class="m-y-8 leading-loose" {
          "The newsletter will contain a summary of the week's posts, as well as more ramblings from me.
          Each week will be different, some weeks I'll share status updates on the projects I'm working on,
          and others I'll share about something I learned or a new tool I've been trying out."
        }

        p class="m-y-8 leading-loose"  {
          "If there is a specific topic you'd like me to write about, check out my Github Sponsors page information about how
          you can sponsor a post! You get to pick the topic, and I'll write about in an upcoming newsletter."
        }

        h2 class="text-2xl pt-16 pb-8" { "Past Newsletters" }

        p."pb-4" { "Coming soon!" }
        p class="leading-loose" {
          "The first scheduled coreyja weekly is set to be released on July 16th 2023.
          Sign up below to get it in your inbox on the 16th, or check back in on the 17th when its posted here."
        }
      }
    })
}
