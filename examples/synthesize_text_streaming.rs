use std::env;

use azure_speech::{stream::StreamExt, synthesizer, Auth};
use tokio_stream::StreamExt as _;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let region = env::var("AZURE_REGION").expect("AZURE_REGION");
    let key = env::var("AZURE_SUBSCRIPTION_KEY").expect("AZURE_SUBSCRIPTION_KEY");

    let auth = Auth::from_subscription(region, key);

    // v2 endpoint is required for text streaming
    let client = synthesizer::Client::connect_v2(auth, synthesizer::Config::new())
        .await
        .expect("connect v2");

    let (writer, mut events) = client
        .synthesize_streaming()
        .await
        .expect("start streaming session");

    // Spawn a task to push text pieces
    let writer_clone = writer.clone();
    let push_task = tokio::spawn(async move {
        let parts = [
            "Input text streaming. ",
            "It reduces TTS latency by sending text incrementally. ",
            "Great for real-time AI agent responses. ",
        ];
        for p in parts {
            writer_clone.write(p).await.expect("write");
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
        writer_clone.finish().await.expect("finish");
    });

    while let Some(evt) = events.next().await {
        match evt {
            Ok(e) => {
                tracing::info!(?e, "event");
            }
            Err(err) => {
                tracing::error!(?err, "error");
                break;
            }
        }
    }

    let _ = push_task.await;
    Ok(())
}
