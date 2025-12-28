use tokio::runtime::Runtime;

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let rt = Runtime::new()?;
    rt.block_on(async {
        arti_chat_daemon::run().await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}

