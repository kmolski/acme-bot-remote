use amiquip::{Connection, Exchange, Publish, Result};
use serde::{Deserialize, Serialize};
use std::env::args;
use std::error::Error;
use std::io;

#[derive(Serialize, Deserialize)]
enum MessageType {
    resume,
    pause,
    stop
}

#[derive(Serialize, Deserialize)]
struct Message {
    r#type: MessageType,
    code: String
}

fn main() -> Result<(), Box<dyn Error>> {
    let argv: Vec<String> = args().collect();
    let uri = &argv[1];

    let mut connection = Connection::insecure_open(&uri)?;
    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);

    let mut code = String::new();
    let mut buf = String::new();

    loop {
        if !code.is_empty() {
            println!("[R]esume, [P]ause, [S]top or [Q]uit?");
            buf.clear();
            io::stdin().read_line(&mut buf)?;
            buf = buf.trim().to_lowercase().to_string();

            let r#type = match buf.chars().next().unwrap() {
                'r' => MessageType::resume,
                'p' => MessageType::pause,
                's' => MessageType::stop,
                'q' => { break; }
                _   => { continue; }
            };

            let message = Message{r#type, code: code.clone()};
            let serialized = serde_json::to_string(&message).unwrap();

            exchange.publish(Publish::new(serialized.as_bytes(), "music_player"))?;
        } else {
            println!("Enter the music player auth code:");
            buf.clear();
            io::stdin().read_line(&mut buf)?;
            buf = buf.trim().to_string();

            if buf.len() != 6 {
                println!("Please enter a 6-digit code!");
                continue;
            } else {
                code = buf.clone();
            }
        }
    }

    Ok(())
}
