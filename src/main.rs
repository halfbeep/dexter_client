use futures_util::{SinkExt, StreamExt};
use rand::Rng;
use serde::Serialize;
use std::time::Duration;
use tokio::time::sleep;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use uuid::Uuid; // Import Uuid for generating unique IDs

#[derive(Serialize, serde::Deserialize, Debug)]
struct Order {
    id: String, // Store the UUID as a String for serialization
    order_type: String,
    price: f64,
    quantity: u32,
}

#[derive(serde::Deserialize)]
struct MessageReceived {
    status: String,
}

async fn stream_random_orders() {
    // Define WebSocket URL
    let url = "ws://127.0.0.1:3030/ws";

    // Connect to the WebSocket server
    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("Connected to ws {}", url);

    // Split the WebSocket stream into the writer (for sending) and reader (for receiving)
    let (mut write, mut read) = ws_stream.split();

    // Spawn a separate task to continuously read messages from the server
    tokio::spawn(async move {
        println!("Starting to read from WebSocket server...");

        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Ok(msgr) = serde_json::from_str::<MessageReceived>(&text) {
                        println!("Received: {} \n", msgr.status);
                    }
                }
                Ok(Message::Binary(bin)) => {
                    println!("Received binary data from server: {:?}", bin);
                }
                Ok(Message::Ping(ping)) => {
                    println!("Received Ping: {:?}", ping);
                }
                Ok(Message::Pong(pong)) => {
                    println!("Received Pong: {:?}", pong);
                }
                Ok(Message::Close(close_frame)) => {
                    if let Some(cf) = close_frame {
                        println!(
                            "Received Close frame: Code: {:?}, Reason: {}",
                            cf.code, cf.reason
                        );
                    } else {
                        println!("Received Close frame with no information.");
                    }
                    break;
                }
                Ok(_) => {
                    println!("Received other type of message");
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                    break;
                }
            }
        }

        println!("Read task completed or WebSocket closed.");
    });

    loop {
        // Generate a unique ID for each order using Uuid and convert it to a String
        let id = Uuid::new_v4().to_string();

        // Generate random order details
        let order_type = if rand::thread_rng().gen_bool(0.5) {
            "Buy".to_string()
        } else {
            "Sell".to_string()
        };

        // Generate a random price between 98.0 and 102.0, explicitly typed as f64
        let price: f64 = rand::thread_rng().gen_range(98.0..=102.0);
        let rounded_price = (price * 10.0).round() / 10.0;

        let quantity: u32 = rand::thread_rng().gen_range(5..=25);

        let order = Order {
            id, // Assign the generated unique ID as a String
            order_type,
            price: rounded_price, // Use the rounded price
            quantity,
        };

        // Serialize the order to JSON
        let order_json = serde_json::to_string(&order).unwrap();

        // Send the order as a WebSocket message
        write
            .send(Message::Text(order_json.clone()))
            .await
            .expect("Failed to send message");

        // Print the sent order
        println!("Sent order: {:?}", order);

        // Sleep for 100 milliseconds before sending the next order
        sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
async fn main() {
    stream_random_orders().await;
}
