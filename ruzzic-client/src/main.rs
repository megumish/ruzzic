use ruzzic::{QUICVersion, Ruzzic, RuzzicInit};
use ruzzic_http3::{GetRequest, GetRequestInit, Http3App, ResponseMessage};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let ruzzic: Ruzzic<_> = RuzzicInit::<Http3App> {
        version: QUICVersion::RFC9000,
        ..Default::default()
    }
    .init()
    .await?;

    let get_request: GetRequest = GetRequestInit {
        url: "https://127.0.0.1:12345",
        ..Default::default()
    }
    .init()
    .await?;

    let message: ResponseMessage = ruzzic
        .send_once(get_request.address().await, get_request.to_message().await)
        .await?
        .to_response()?;

    println!("{message:?}");

    Ok(())
}
