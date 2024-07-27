#!/usr/bin/env -S cargo +nightly -Zscript

//! ```cargo
//! [dependencies]
//! git2 = { version = "0.18.1" }
//! miette = { version = "5.9.0", features = ["fancy"] }
//! glob = "0.3"
//! ```

use git2::{Error, Repository};
use glob::glob;
use miette::IntoDiagnostic;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn main() -> color_eyre::Result<()> {
    let repo_path = "/Users/coreyja/Projects/coreyja.com";
    let file_path = "screenshots/4k.png";
    let out_dir = "out";
    let repo = Repository::open(repo_path)?;

    let mut revwalk = repo.revwalk()?;
    revwalk.push_head()?;

    std::fs::create_dir_all(out_dir)?;

    let mut ids = revwalk.collect::<Result<Vec<_>, Error>>()?;

    ids.reverse();

    for (i, id) in ids.into_iter().enumerate() {
        // let id = id?;
        let commit = repo.find_commit(id)?;
        let tree = commit.tree()?;

        let Ok(object) = tree.get_path(Path::new(file_path)) else {
            continue;
        };
        let Ok(object) = object.to_object(&repo) else {
            continue;
        };

        if let Some(blob) = object.as_blob() {
            let content = blob.content();
            let mut file = File::create(format!("{out_dir}/image_version_{:0>5}.png", i))?;
            file.write_all(content)?;
        }
    }

    let frame_glob = format!("{out_dir}/image_version_*.png");

    Command::new("ffmpeg")
        .arg("-framerate")
        .arg("8")
        .arg("-pattern_type")
        .arg("glob")
        .arg("-i")
        .arg(&frame_glob)
        .arg(format!("{out_dir}/video.mp4"))
        .spawn()?
        .wait()?;

    for entry in glob(&frame_glob)? {
        match entry {
            Ok(path) => {
                fs::remove_file(path)?;
            }
            Err(e) => println!("Error reading file: {:?}", e),
        }
    }

    Ok(())
}
