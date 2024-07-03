use tokio::process::Command;

pub async fn say(message: &str) -> color_eyre::Result<()> {
    let c = Command::new("say").arg(message).spawn()?;
    c.wait_with_output().await?;

    Ok(())
}

pub async fn say_loop(mut reciever: tokio::sync::mpsc::Receiver<String>) -> color_eyre::Result<()> {
    while let Some(message) = reciever.recv().await {
        say(&message).await?;
    }

    unreachable!("The say channel should never close")
}
