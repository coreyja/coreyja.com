use std::{fs::File, io::Write, process::Command};

use miette::{IntoDiagnostic, Result};
use posts::projects::Projects;
use rsa::pkcs8::{EncodePrivateKey, EncodePublicKey};

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
        eprintln!("No fly app name found for {}", project_slug);
        std::process::exit(1);
    };

    println!(
        "Creating public/private key pair for {}",
        project.frontmatter.title
    );

    let mut rng = rand::thread_rng();
    let bits = 2048;
    let priv_key = rsa::RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = rsa::RsaPublicKey::from(&priv_key);

    let private_pem = priv_key.to_pkcs8_pem(Default::default()).unwrap();
    let public_pem = pub_key.to_public_key_pem(Default::default()).unwrap();

    println!("Keys generated! ✅");

    let mut public_key_file = File::create(format!("projects/{}.pub.pem", project_slug)).unwrap();

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
