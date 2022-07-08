use amiquip::{Connection, Exchange, Publish, Result};
use serde::{Deserialize, Serialize};
use std::env::args;
use std::error::Error;
use std::io;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum MessageType {
    Resume,
    Pause,
    Stop,
}

#[derive(Serialize, Deserialize)]
struct Message {
    op: MessageType,
    code: String,
}

const CODE_LENGTH: usize = 6;

fn read_line() -> io::Result<String> {
    let mut buf = String::new();
    io::stdin().read_line(&mut buf)?;
    Ok(buf.trim().to_lowercase())
}

fn main() -> Result<(), Box<dyn Error>> {
    let argv: Vec<String> = args().collect();
    let uri = &argv[1];

    let mut connection = Connection::insecure_open(uri)?;
    let channel = connection.open_channel(None)?;
    let exchange = Exchange::direct(&channel);

    let mut code: Option<String> = None;

    loop {
        if let Some(auth_code) = &code {
            println!("[R]esume, [P]ause, [S]top, [C]hange code or [Q]uit?");

            let selection = read_line()?;
            let op = match selection.as_str() {
                "r" => MessageType::Resume,
                "p" => MessageType::Pause,
                "s" => MessageType::Stop,
                "c" => {
                    code = None;
                    continue;
                }
                "q" => {
                    break;
                }
                _ => {
                    continue;
                }
            };

            let message = Message {
                op,
                code: auth_code.clone(),
            };
            let serialized = serde_json::to_string(&message).unwrap();

            exchange.publish(Publish::new(serialized.as_bytes(), "music_player"))?;
        } else {
            println!("Enter the music player auth code:");

            let new_code = read_line()?;
            if new_code.len() != CODE_LENGTH {
                println!("Please enter a {CODE_LENGTH}-digit code!");
                continue;
            } else {
                code = Some(new_code);
            }
        }
    }

    Ok(())
}
