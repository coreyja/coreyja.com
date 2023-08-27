use irc::proto::message;
use miette::IntoDiagnostic;
use tokio::process::Command;

pub async fn say(message: &str) -> miette::Result<()> {
    let c = Command::new("say").arg(message).spawn().into_diagnostic()?;
    c.wait_with_output().await.into_diagnostic()?;

    Ok(())
}

pub async fn say_loop(mut reciever: tokio::sync::mpsc::Receiver<String>) -> miette::Result<()> {
    loop {
        let message = reciever.recv().await;

        let Some(msg) = message else {
          return Err(miette::miette!("Say channel has been closed"));
        };

        say(&msg).await?;
    }
}
