use std::path::PathBuf;

use clap::Args;
use tokio::io::AsyncWriteExt;

use crate::*;

#[derive(Debug, Args)]
pub(crate) struct Blogify {
    #[clap(long)]
    bucket: String,
    #[clap(long)]
    prefix: String,
}

impl Blogify {
    pub async fn blogify(&self) -> Result<()> {
        let config = ::aws_config::load_from_env().await;
        let client = s3::Client::new(&config);

        let objects = get_all_objects_for_bucket(client, &self.bucket, &self.prefix).await?;

        let mut video_infos = objects
            .iter()
            .filter(|x| match x.key() {
                Some(k) => k.ends_with(".yt_v2.txt"),
                None => false,
            })
            .collect::<Vec<_>>();

        video_infos.sort_by_key(|x| -x.size);

        dbg!(video_infos.len());

        let mut videos = vec![];
        for info in video_infos {
            let key_path = info.key().unwrap();
            let key_path = key_path.strip_suffix(".yt_v2.txt").unwrap();
            let uploaded_path = format!("{}.upload.txt", key_path);
            let video_path = format!("{}.mkv", key_path);

            let client = s3::Client::new(&config);
            let resp = client
                .get_object()
                .bucket(&self.bucket)
                .key(info.key.as_ref().unwrap())
                .send()
                .await
                ?;
            let youtube_info_data = resp
                .body
                .collect()
                .await
                .expect("error reading data")
                .into_bytes();
            let youtube_info_data =
                String::from_utf8(youtube_info_data.to_vec()).expect("invalid utf8");

            let mut s = youtube_info_data.split('\n');
            let title = s.next().unwrap();
            let description = s.collect::<Vec<_>>().join("\n");

            // let title = title.strip_prefix('"').unwrap_or(title);
            // let title = title.strip_suffix('"').unwrap_or(title);
            let title = title.replace('"', "");
            let title = title.replace("Title: ", "");
            let title = title.trim();

            let upload_file_resp = s3::Client::new(&config)
                .get_object()
                .bucket(&self.bucket)
                .key(&uploaded_path)
                .send()
                .await;

            let upload_url = if let Ok(resp) = upload_file_resp {
                Some(resp.body.collect().await.expect("error reading data"))
            } else {
                None
            };

            let s3_path = video_path;

            let date = {
                let mut p = PathBuf::from(info.key().unwrap());
                p.set_extension("");
                let date = p.file_name().unwrap().to_str().unwrap();
                let date = date.split(' ').next().unwrap();

                chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d").unwrap()
            };

            videos.push(PastStream {
                title: title.to_string(),
                description: description.to_string(),
                youtube_url: upload_url.map(|x| String::from_utf8(x.to_vec()).unwrap()),
                s3_url: s3_path,
                date,
            });
        }

        let uploaded = videos
            .iter()
            .filter(|x| x.youtube_url.is_some())
            .collect::<Vec<_>>();
        dbg!(&uploaded);

        tokio::fs::create_dir_all("./past_streams")
            .await
            ?;

        for stream in videos {
            let mut file = tokio::fs::File::create(format!(
                "./past_streams/{}.md",
                stream.date.format("%Y-%m-%d")
            ))
            .await
            ?;

            let youtube_url = stream
                .youtube_url
                .as_deref()
                .map(|url| format!("youtube_url: \"{}\"", url.trim()))
                .unwrap_or_else(String::new);

            let frontmatter = format!(
                r#"title: "{}"
date: {}
s3_url: "{}"
{}
"#,
                stream.title,
                stream.date.format("%Y-%m-%d"),
                stream.s3_url,
                youtube_url
            );
            let frontmatter = format!("---\n{}\n---", frontmatter.trim());
            let content = format!("{}\n{}\n", frontmatter, stream.description);

            println!("{}", content);

            file.write_all(content.as_bytes()).await?;
            file.flush().await?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct PastStream {
    title: String,
    description: String,
    youtube_url: Option<String>,
    s3_url: String,
    date: chrono::NaiveDate,
}
