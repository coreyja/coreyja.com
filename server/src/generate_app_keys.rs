use std::{fs::File, io::Write, process::Command};

use age::secrecy::zeroize::Zeroizing;
use miette::{IntoDiagnostic, Result};
use posts::{
    projects::{FrontMatterWithKey, Projects},
    Post,
};
use rand::rngs::ThreadRng;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey, LineEnding};
use tokio::fs::create_dir_all;

fn generate_keys_for_project(
    rng: &mut ThreadRng,
    project: &Post<FrontMatterWithKey>,
    fly_app_name: &str,
) -> Result<()> {
    let project_slug = project.slug().unwrap();
    println!(
        "Creating public/private key pair for {}",
        project.frontmatter.title
    );

    let (private_pem, public_pem) = gen_keys(rng)?;

    let mut public_key_file = File::create(format!("projects/{project_slug}/key.pub.pem")).unwrap();
    public_key_file
        .write_all(public_pem.as_bytes())
        .into_diagnostic()?;

    Command::new("fly")
        .arg("secrets")
        .args(["--app", &fly_app_name])
        .arg("set")
        .arg(format!("AUTH_PRIVATE_KEY={}", *private_pem))
        .status()
        .into_diagnostic()?;

    Ok(())
}

fn generate_testing_keys_for_project(
    rng: &mut ThreadRng,
    project: &Post<FrontMatterWithKey>,
) -> Result<()> {
    let project_slug = project.slug().unwrap();

    let (testing_private_pem, testing_public_pem) = gen_keys(rng)?;

    let mut testing_public_key_file =
        File::create(format!("projects/{project_slug}/testing.pub.pem")).unwrap();
    testing_public_key_file
        .write_all(testing_public_pem.as_bytes())
        .into_diagnostic()?;

    let mut testing_private_key_file =
        File::create(format!("projects/{project_slug}/testing.private.pem")).unwrap();
    testing_private_key_file
        .write_all(testing_private_pem.as_bytes())
        .into_diagnostic()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = std::env::args().collect::<Vec<_>>();
    let project_slug = args.get(1).unwrap_or_else(|| {
        eprintln!("Usage: {} <project_slug>", args[0]);
        std::process::exit(1);
    });

    let projects = Projects::from_static_dir()?;
    let project = projects
        .projects
        .iter()
        .find(|p| p.slug().unwrap() == project_slug)
        .unwrap();

    let Some(fly_app_name) = project.frontmatter.fly_app_name.clone() else {
        eprintln!("No fly app name found for {project_slug}");
        std::process::exit(1);
    };

    create_dir_all(format!("projects/{project_slug}"))
        .await
        .into_diagnostic()?;

    let mut rng = rand::thread_rng();
    generate_keys_for_project(&mut rng, project, &fly_app_name)?;

    generate_testing_keys_for_project(&mut rng, project)?;

    Ok(())
}

fn gen_keys(rng: &mut ThreadRng) -> Result<(Zeroizing<String>, String)> {
    let bits = 2048;
    let priv_key = rsa::RsaPrivateKey::new(rng, bits).expect("failed to generate a key");
    let pub_key = rsa::RsaPublicKey::from(&priv_key);

    let private_pem = priv_key
        .to_pkcs8_pem(LineEnding::default())
        .into_diagnostic()?;
    let public_pem = pub_key
        .to_public_key_pem(LineEnding::default())
        .into_diagnostic()?;

    Ok((private_pem, public_pem))
}
