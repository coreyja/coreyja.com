use std::fmt::Display;

use include_dir::{include_dir, Dir};
use miette::{Context, Diagnostic, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{title::Title, MarkdownAst, Post};

pub(crate) static PROJECTS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../projects");

#[derive(Debug, Clone)]
pub struct Projects {
    pub projects: Vec<Project>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum ProjectStatus {
    Active,
    Maintenance,
    OnIce,
    Complete,
    Archived,
}

impl Display for ProjectStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectStatus::Active => f.write_str("Active"),
            ProjectStatus::Maintenance => f.write_str("Maintenance"),
            ProjectStatus::OnIce => f.write_str("On Ice"),
            ProjectStatus::Complete => f.write_str("Complete"),
            ProjectStatus::Archived => f.write_str("Archived"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct FrontMatter {
    pub title: String,
    pub subtitle: Option<String>,
    pub repo: String,
    pub youtube_playlist: Option<String>,
    pub parent_project: Option<String>,
    pub status: ProjectStatus,
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
            .wrap_err("One of the Projects failed to parse")?;

        Ok(Self { projects })
    }

    pub fn validate(&self) -> Result<()> {
        println!("Validating {} Projects", self.projects.len());
        let mut errs = vec![];
        for stream in &self.projects {
            println!(
                "Validating {} from {}...",
                stream.frontmatter.title,
                stream.path.display()
            );

            let validation_reslut = stream.validate(self);

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

    pub fn by_title(&self) -> Vec<Project> {
        let mut projects = self.projects.clone();
        projects.sort_by(|a, b| a.frontmatter.title.cmp(&b.frontmatter.title));
        projects
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

    pub fn validate(&self, projects: &Projects) -> Result<()> {
        if self.frontmatter.title.chars().count() >= 100 {
            return Err(miette::miette!(
                "Title is too long: {}",
                self.frontmatter.title.clone(),
            ));
        }

        if self.slug().is_err() {
            return Err(miette::miette!(
                "Could not get slug from path: {}",
                self.path.display(),
            ));
        };

        if let Some(parent_slug) = &self.frontmatter.parent_project {
            if !projects
                .projects
                .iter()
                .any(|p| p.slug().unwrap() == parent_slug)
            {
                return Err(miette::miette!("Parent project not found: {}", parent_slug,));
            }
        }

        Ok(())
    }

    pub fn slug(&self) -> Result<&str> {
        let stem = self
            .path
            .file_stem()
            .ok_or_else(|| miette::miette!("No file stem for {:?}", self.path))?;
        let s = stem
            .to_str()
            .ok_or_else(|| miette::miette!("Couldn't create a String from {stem:?}"))?;

        Ok(s)
    }

    pub fn relative_link(&self) -> Result<String> {
        Ok(format!("/projects/{}", self.slug()?))
    }
}

#[derive(Debug, Error, Diagnostic)]
#[error("The were errors validating the Projects")]
struct ValidationError {
    #[related]
    others: Vec<miette::Report>,
}

impl Title for Project {
    fn title(&self) -> &str {
        &self.frontmatter.title
    }
}