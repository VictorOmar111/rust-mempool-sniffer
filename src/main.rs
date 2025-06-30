use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use tokio_tungstenite::connect_async;
use url::Url;
use reqwest::Client;
use log::{info, error};

#[tokio::main]
async fn main() {
    env_logger::init();

    let ws_url = "wss://eth.llamarpc.com";
    let url = Url::parse(ws_url).unwrap();
    let (ws_stream, _) = connect_async(url).await.expect("❌ Failed to connect");
    info!("✅ Connected to {}", ws_url);

    let (mut write, mut read) = ws_stream.split();

    let subscribe_msg = json!({
        "id": 1,
        "method": "eth_subscribe",
        "params": ["newPendingTransactions"]
    })
    .to_string();

    write.send(tokio_tungstenite::tungstenite::Message::Text(subscribe_msg)).await.unwrap();
    info!("🔄 Subscribed to pending transactions...");

    let client = Client::new();

    while let Some(msg) = read.next().await {
        if let Ok(tokio_tungstenite::tungstenite::Message::Text(text)) = msg {
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(params) = parsed.get("params") {
                    if let Some(result) = params.get("result") {
                        if let Some(tx_hash) = result.as_str() {
                            info!("📥 New pending tx hash: {}", tx_hash);
                            fetch_tx_details(&client, tx_hash).await;
                        }
                    }
                }
            }
        }
    }
}

async fn fetch_tx_details(client: &Client, tx_hash: &str) {
    let rpc_url = "https://eth.llamarpc.com";
    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "eth_getTransactionByHash",
        "params": [tx_hash]
    });

    match client.post(rpc_url).json(&payload).send().await {
        Ok(resp) => {
            match resp.json::<serde_json::Value>().await {
                Ok(resp_json) => {
                    if let Some(result) = resp_json.get("result") {
                        let from = result.get("from").and_then(|v| v.as_str()).unwrap_or("N/A");
                        let to = result.get("to").and_then(|v| v.as_str()).unwrap_or("N/A");
                        let value_hex = result.get("value").and_then(|v| v.as_str()).unwrap_or("0x0");
                        let input = result.get("input").and_then(|v| v.as_str()).unwrap_or("");

                        info!("🔹 From: {}", from);
                        info!("🔹 To: {}", to);
                        info!("🔹 Value: {}", hex_to_eth(value_hex));
                        info!("🔹 Input (first 10): {}", &input[..10.min(input.len())]);
                        info!("----------------------------------------");
                    }
                }
                Err(e) => error!("❌ Error parsing JSON response: {}", e),
            }
        }
        Err(e) => error!("❌ Error sending RPC request: {}", e),
    }
}

fn hex_to_eth(hex: &str) -> String {
    if let Ok(val) = u128::from_str_radix(&hex.trim_start_matches("0x"), 16) {
        let eth_val = val as f64 / 1e18;
        format!("{:.6} ETH", eth_val)
    } else {
        "0 ETH".to_string()
    }
}
