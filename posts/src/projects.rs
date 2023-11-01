use include_dir::{include_dir, Dir};
use miette::{Context, Diagnostic, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{MarkdownAst, Post};

pub(crate) static PROJECTS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../projects");

#[derive(Debug, Clone)]
pub struct Projects {
    pub projects: Vec<Project>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct FrontMatter {
    pub title: String,
    pub subtitle: String,
    pub repo: String,
    pub youtube_playlist: Option<String>,
}

pub type Project = Post<FrontMatter>;

impl Projects {
    pub fn from_static_dir() -> Result<Self> {
        Self::from_dir(&PROJECTS_DIR)
    }

    pub fn from_dir(dir: &Dir) -> Result<Self> {
        let projects = dir
            .find("**/*.md")
            .into_diagnostic()?
            .filter_map(|e| e.as_file())
            .map(Project::from_file)
            .collect::<Result<Vec<_>>>()
            .wrap_err("One of the TILs failed to parse")?;

        Ok(Self { projects })
    }

    pub fn validate(&self) -> Result<()> {
        println!("Validating {} Streams", self.projects.len());
        let mut errs = vec![];
        for stream in &self.projects {
            println!(
                "Validating {} from {}...",
                stream.frontmatter.title,
                stream.path.display()
            );

            let validation_reslut = stream.validate();

            if let Err(e) = validation_reslut {
                errs.push(e);
            }
        }

        if !errs.is_empty() {
            return Err(ValidationError { others: errs }.into());
        }

        println!("Streams Valid! ✅");

        Ok(())
    }
}

impl Project {
    fn from_file(file: &include_dir::File) -> Result<Self> {
        let ast = MarkdownAst::from_file(file)?;
        let metadata: FrontMatter = ast.frontmatter()?;
        let path = file.path().to_owned();

        Ok(Self {
            ast,
            path,
            frontmatter: metadata,
        })
    }

    pub fn validate(&self) -> Result<()> {
        if self.frontmatter.title.chars().count() >= 100 {
            return Err(miette::miette!(
                "Title is too long: {}",
                self.frontmatter.title.clone(),
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("The were errors validating the Projects")]
struct ValidationError {
    #[related]
    others: Vec<miette::Report>,
}
