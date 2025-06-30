use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::connect_async;
use url::Url;

#[tokio::main]
async fn main() {
    env_logger::init();

    let ws_url = "wss://eth.llamarpc.com"; // Cambia a tu RPC WS privado si lo deseas

    let url = Url::parse(ws_url).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("✅ Connected to {}", ws_url);

    let (mut write, mut read) = ws_stream.split();

    let subscribe_msg = json!({
        "id": 1,
        "method": "eth_subscribe",
        "params": ["newPendingTransactions"]
    })
    .to_string();

    write.send(tokio_tungstenite::tungstenite::Message::Text(subscribe_msg)).await.unwrap();
    println!("🔄 Subscribed to pending transactions...");

    while let Some(msg) = read.next().await {
        match msg {
            Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(params) = parsed.get("params") {
                        if let Some(result) = params.get("result") {
                            println!("📥 New pending tx hash: {}", result.as_str().unwrap());
                        }
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                break;
            }
        }
    }
}
