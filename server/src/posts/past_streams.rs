use chrono::NaiveDate;
use include_dir::{include_dir, Dir, File};
use miette::{Context, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};

use super::{
    date::{ByRecency, PostedOn},
    title::Title,
    MarkdownAst, Post,
};

pub(crate) static TIL_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../past_streams");

#[derive(Debug, Clone)]
pub(crate) struct PastStreams {
    pub(crate) streams: Vec<PastStream>,
}

pub(crate) type PastStream = Post<FrontMatter>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub(crate) struct FrontMatter {
    pub title: String,
    pub date: NaiveDate,
    pub s3_url: String,
    pub youtube_url: Option<String>,
}

impl PostedOn for FrontMatter {
    fn posted_on(&self) -> chrono::NaiveDate {
        self.date
    }
}

impl Title for FrontMatter {
    fn title(&self) -> &str {
        &self.title
    }
}

impl PastStreams {
    pub fn from_static_dir() -> Result<Self> {
        Self::from_dir(&TIL_DIR)
    }

    pub fn from_dir(dir: &Dir) -> Result<Self> {
        let streams = dir
            .find("**/*.md")
            .into_diagnostic()?
            .filter_map(|e| e.as_file())
            .map(PastStream::from_file)
            .collect::<Result<Vec<_>>>()
            .wrap_err("One of the TILs failed to parse")?;

        Ok(Self { streams })
    }

    pub fn validate(&self) -> Result<()> {
        println!("Validating {} Streams", self.streams.len());
        for stream in &self.streams {
            println!(
                "Validating {} from {}...",
                stream.frontmatter.title,
                stream.path.display()
            );

            stream.validate()?;
        }
        println!("Streams Valid! ✅");

        Ok(())
    }

    pub fn by_recency(&self) -> Vec<&PastStream> {
        self.streams.by_recency()
    }
}

impl PastStream {
    fn from_file(file: &File) -> Result<Self> {
        let ast = MarkdownAst::from_file(file)?;
        let metadata: FrontMatter = ast.frontmatter()?;
        let path = file.path().to_owned();

        Ok(Self {
            ast,
            path,
            frontmatter: metadata,
        })
    }

    fn validate(&self) -> Result<()> {
        Ok(())
    }
}
