use ruzzic::{simple_app::SimpleAppMessage, Ruzzic, RuzzicInit, SimpleApp};
use ruzzic_common::QuicVersion;
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

    let mut ruzzic_server = ruzzic.server().await;

    loop {
        let message = ruzzic_server.next().await;
        println!("{message:?}");
    }

    Ok(())
}
