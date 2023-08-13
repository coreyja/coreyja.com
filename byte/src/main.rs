pub use miette::Result;
use tracing_common::setup_tracing;

#[tokio::main]
async fn main() -> Result<()> {
    setup_tracing()?;

    println!("Hello, world!");

    Ok(())
}
