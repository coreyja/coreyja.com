use maud::{html, Markup, Render};

pub(crate) struct YoutubeVideo {
    pub(crate) title: String,
    pub(crate) description: Option<String>,
    pub(crate) youtube_video_id: String,
    pub(crate) external_youtube_id: String,
    pub(crate) thumbnail_url: Option<String>,
    pub(crate) published_at: Option<chrono::NaiveDateTime>,
}

pub(crate) struct VideoList(pub(crate) Vec<YoutubeVideo>);

impl Render for VideoList {
    fn render(&self) -> Markup {
        html! {
          ul {
            @for video in &self.0 {
              li class="my-4" {
                a href=("#") {
                    span class="text-subtitle text-sm inline-block w-[80px]" { (video.published_at.unwrap()) }
                    " "

                    (video.title)
                }
              }
            }
          }
        }
    }
}
