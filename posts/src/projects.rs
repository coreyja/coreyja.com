use std::fmt::Display;

use include_dir::{include_dir, Dir};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

use crate::{title::Title, MarkdownAst, Post};
use color_eyre::{eyre::Context, Result};

pub(crate) static PROJECTS_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../projects");

#[derive(Debug, Clone)]
pub struct Projects {
    pub projects: Vec<Project>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Copy, Hash, Eq, PartialOrd, Ord)]
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
struct FrontMatter {
    pub title: String,
    pub subtitle: Option<String>,
    pub repo: String,
    pub youtube_playlist: Option<String>,
    pub parent_project: Option<String>,
    pub status: ProjectStatus,
    pub login_callback: Option<String>,
    pub local_port: Option<u16>,
    pub fly_app_name: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct FrontMatterWithKey {
    pub title: String,
    pub subtitle: Option<String>,
    pub repo: String,
    pub youtube_playlist: Option<String>,
    pub parent_project: Option<String>,
    pub status: ProjectStatus,
    pub login_callback: Option<String>,
    pub local_port: Option<u16>,
    pub fly_app_name: Option<String>,
    pub auth_public_key: Option<String>,
}

impl FrontMatterWithKey {
    pub(self) fn from_frontmatter(frontmatter: FrontMatter, pub_key: Option<String>) -> Self {
        Self {
            title: frontmatter.title,
            subtitle: frontmatter.subtitle,
            repo: frontmatter.repo,
            youtube_playlist: frontmatter.youtube_playlist,
            parent_project: frontmatter.parent_project,
            status: frontmatter.status,
            login_callback: frontmatter.login_callback,
            local_port: frontmatter.local_port,
            fly_app_name: frontmatter.fly_app_name,
            auth_public_key: pub_key,
        }
    }
}

pub type Project = Post<FrontMatterWithKey>;

impl Projects {
    pub fn from_static_dir() -> Result<Self> {
        Self::from_dir(&PROJECTS_DIR)
    }

    pub fn from_dir(dir: &Dir) -> Result<Self> {
        let projects = dir
            .find("**/*.md")?
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

            let validation_result = stream.validate(self);

            if let Err(e) = validation_result {
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

        let mut pub_key_path = file.path().to_owned();
        pub_key_path = pub_key_path.with_extension("");

        let key_filename =
            std::env::var("AUTH_KEY_FILENAME").unwrap_or_else(|_| "key.pub.pem".to_string());
        pub_key_path.push(key_filename);

        let pub_key_path = &pub_key_path;
        // THIS IS A HACK!
        // it will only work from static but thats the only way i use it
        let pub_key_file = PROJECTS_DIR.get_file(pub_key_path);
        let pub_key = pub_key_file.map(|f| f.contents_utf8().unwrap().to_string());

        Ok(Self {
            ast,
            path,
            frontmatter: FrontMatterWithKey::from_frontmatter(metadata, pub_key),
        })
    }

    pub fn validate(&self, projects: &Projects) -> Result<()> {
        if self.frontmatter.title.chars().count() >= 100 {
            return Err(color_eyre::eyre::eyre!(
                "Title is too long: {}",
                self.frontmatter.title.clone(),
            ));
        }

        if self.slug().is_err() {
            return Err(color_eyre::eyre::eyre!(
                "Could not get slug from path: {}",
                self.path.display(),
            ));
        }

        if let Some(parent_slug) = &self.frontmatter.parent_project {
            if !projects
                .projects
                .iter()
                .any(|p| p.slug().unwrap() == parent_slug)
            {
                return Err(color_eyre::eyre::eyre!(
                    "Parent project not found: {}",
                    parent_slug,
                ));
            }
        }

        Ok(())
    }

    pub fn slug(&self) -> Result<&str> {
        let stem = self
            .path
            .file_stem()
            .ok_or_else(|| color_eyre::eyre::eyre!("No file stem for {:?}", self.path))?;
        let s = stem
            .to_str()
            .ok_or_else(|| color_eyre::eyre::eyre!("Couldn't create a String from {stem:?}"))?;

        Ok(s)
    }

    pub fn relative_link(&self) -> Result<String> {
        Ok(format!("/projects/{}", self.slug()?))
    }

    pub fn login_callback(&self) -> Result<Url> {
        let login_callback = self.frontmatter.login_callback.clone().ok_or_else(|| {
            let slug = self.slug().map_or_else(
                |e| format!("unknown got error {e}"),
                std::string::ToString::to_string,
            );
            color_eyre::eyre::eyre!("No login_callback found for {}", slug)
        })?;

        #[allow(unused_mut)]
        let mut login_callback = login_callback.parse::<url::Url>()?;

        #[cfg(feature = "test_auth")]
        {
            println!("Inside conditional block");
            let local_port = self.local_port()?;
            println!("After getting local port");

            login_callback.set_host(Some("localhost"))?;
            println!("After setting host");

            login_callback
                .set_port(Some(local_port))
                .map_err(|()| color_eyre::eyre::eyre!("Port could not be set"))?;
            println!("After setting port");

            login_callback
                .set_scheme("http")
                .map_err(|()| color_eyre::eyre::eyre!("Scheme could not be set"))?;
            println!("After setting scheme");
        }
        println!("After conditional block");

        Ok(login_callback)
    }

    pub fn local_port(&self) -> Result<u16> {
        self.frontmatter
            .local_port
            .ok_or_else(|| color_eyre::eyre::eyre!("No local_port found"))
    }
}

#[derive(Debug, Error)]
#[error("The were errors validating the Projects")]
struct ValidationError {
    others: Vec<color_eyre::Report>,
}

impl Title for Project {
    fn title(&self) -> &str {
        &self.frontmatter.title
    }
}
