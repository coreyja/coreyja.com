use miette::IntoDiagnostic;
use tokio::process::Command;

pub async fn say(message: &str) -> miette::Result<()> {
    let c = Command::new("say").arg(message).spawn().into_diagnostic()?;
    c.wait_with_output().await.into_diagnostic()?;

    Ok(())
}

pub async fn say_loop(mut reciever: tokio::sync::mpsc::Receiver<String>) -> miette::Result<()> {
    while let Some(message) = reciever.recv().await {
        say(&message).await?;
    }

    unreachable!("The say channel should never close")
}
