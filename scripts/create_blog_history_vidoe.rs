#!/usr/bin/env -S cargo +nightly -Zscript

//! ```cargo
//! [dependencies]
//! git2 = { version = "0.18.1" }
//! miette = { version = "5.9.0", features = ["fancy"] }
//! ```

use git2::{Error, Repository};
use miette::IntoDiagnostic;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() -> miette::Result<()> {
    let repo_path = "/Users/coreyja/Projects/coreyja.com";
    let file_path = "screenshots/4k.png";
    let out_dir = "out";
    let repo = Repository::open(repo_path).into_diagnostic()?;

    let mut revwalk = repo.revwalk().into_diagnostic()?;
    revwalk.push_head().into_diagnostic()?;

    std::fs::create_dir_all(out_dir).into_diagnostic()?;

    let mut ids = revwalk
        .collect::<Result<Vec<_>, Error>>()
        .into_diagnostic()?;

    ids.reverse();

    for (i, id) in ids.into_iter().enumerate() {
        // let id = id.into_diagnostic()?;
        let commit = repo.find_commit(id).into_diagnostic()?;
        let tree = commit.tree().into_diagnostic()?;

        let Ok(object) = tree.get_path(Path::new(file_path)) else {
            continue;
        };
        let Ok(object) = object.to_object(&repo) else {
            continue;
        };

        if let Some(blob) = object.as_blob() {
            let content = blob.content();
            let mut file =
                File::create(format!("{out_dir}/image_version_{:0>5}.png", i)).into_diagnostic()?;
            file.write_all(content).into_diagnostic()?;
        }
    }

    Command::new("ffmpeg")
        .arg("-framerate")
        .arg("16")
        .arg("-pattern_type")
        .arg("glob")
        .arg("-i")
        .arg(format!("{out_dir}/image_version_*.png"))
        .arg(format!("{out_dir}/video.mp4"))
        .spawn()
        .into_diagnostic()?
        .wait()
        .into_diagnostic()?;

    Ok(())
}
