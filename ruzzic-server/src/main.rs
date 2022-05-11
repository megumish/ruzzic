use ruzzic::{QuicVersion, Ruzzic, RuzzicInit, SimpleApp};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ruzzic: Ruzzic<_> = RuzzicInit::<SimpleApp> {
        support_versions: vec![QuicVersion::Rfc9000],
        self_addr: "0.0.0.0:12345",
        ..Default::default()
    }
    .init()
    .await?;

    let ruzzic_server = ruzzic.server().await;

    while let Some(message) = ruzzic_server.next().await? {
        println!("{message:?}");
    }

    Ok(())
}
