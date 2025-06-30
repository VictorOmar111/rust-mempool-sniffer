use std::env;
use tokio::net::TcpStream;
use websocket::{ClientBuilder, OwnedMessage};

#[tokio::main]
async fn main() {
    env_logger::init();

    let ws_url = "wss://mainnet.infura.io/ws/v3/YOUR_PROJECT_ID"; // o tu RPC privado
    println!("Connecting to {}", ws_url);

    let client = ClientBuilder::new(ws_url)
        .unwrap()
        .async_connect_secure(None)
        .unwrap();

    let (mut receiver, mut sender) = client.split().unwrap();

    let subscription = r#"{
        "id": 1,
        "method": "eth_subscribe",
        "params": ["newPendingTransactions"]
    }"#;

    sender.send_message(&OwnedMessage::Text(subscription.to_string())).unwrap();

    println!("Subscribed to pending transactions...");

    for message in receiver.incoming_messages() {
        let message = message.unwrap();

        match message {
            OwnedMessage::Text(txt) => {
                println!("New tx hash: {}", txt);
                // Aquí puedes parsear el JSON y filtrar hashes o métodos de interés
            }
            OwnedMessage::Close(_) => {
                println!("Connection closed");
                break;
            }
            _ => (),
        }
    }
}
