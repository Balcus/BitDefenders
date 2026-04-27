use anyhow::Context;
use bitdefender::commands::{shared::{EndMatchArgs, MoveArgs, ShootArgs}, start_match::StartMatch, start_turn::StartTurn};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    command: Command,
    args: serde_json::Value,
}

impl WebSocketMessage {
    pub fn empty(command: Command) -> Self {
        Self {
            command,
            args: serde_json::Value::Null,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Command {
    Hello,
    Login,
    Error,
    Ready,
    Practice,
    StartMatch,
    StartTurn,
    Move,
    EndMatch,
}

async fn send_command<
    S: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
>(
    write: &mut S,
    msg: WebSocketMessage,
) -> anyhow::Result<()> {
    let msg_deserialized = serde_json::to_string(&msg).context("serialize message")?;
    write
        .send(Message::Text(msg_deserialized.into()))
        .await
        .context("send message")?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let url = "wss://bitdefenders.cvjd.me/ws";
    let (ws, _) = connect_async(url).await.unwrap();
    let (mut write, mut read) = ws.split();
    let mut current_turn: StartTurn;
    let mut me_id: i32 = 0;

    println!("connected");

    while let Some(msg) = read.next().await {
        let msg = msg.unwrap();

        let text = match msg {
            Message::Text(text) => text,
            Message::Ping(payload) => {
                write.send(Message::Pong(payload)).await.unwrap();
                continue;
            }
            Message::Pong(_) => {
                println!("pong");
                continue;
            }
            Message::Binary(_) => {
                println!("binary message ignored");
                continue;
            }
            Message::Close(frame) => {
                println!("closed: {frame:?}");
                break;
            }
            Message::Frame(_) => continue,
        };

        let message: WebSocketMessage = serde_json::from_str(&text).unwrap();
        println!("{message:?}");
        match message.command {
            Command::Hello => {
                // Send login
                if let Err(e) = send_command(
                    &mut write,
                    WebSocketMessage {
                        command: Command::Login,
                        args: serde_json::json!({"version": 1, "name": "Balcus"}),
                    },
                )
                .await
                {
                    println!("Failed to send login command: {e}");
                    break;
                }
            }
            Command::Login => {
                panic!("What are you doing here?");
            }
            Command::Error => {
                println!("Error: {message:?}");
                break;
            }
            Command::Ready => {
                println!("You are ready to play!");
                send_command(&mut write, WebSocketMessage::empty(Command::Practice)).await?;
            }
            Command::Practice => {
                panic!("Nope");
            }
            Command::StartMatch => {
                let response: StartMatch = serde_json::from_value(message.args).unwrap();
                me_id = response.your_player_id;
            }
            Command::StartTurn => {
                current_turn = serde_json::from_value(message.args).unwrap();
                println!("{:?}", current_turn);
                let args = MoveArgs {
                    hero_id: me_id,
                    x: -1,
                    y: 1,
                };
                send_command(
                    &mut write,
                    WebSocketMessage {
                        command: Command::Move,
                        args: serde_json::to_value(args.clone()).unwrap(),
                    },
                )
                .await?;
                let args2 = ShootArgs {
                    hero_id: 1,
                    x: -1,
                    y: -1,
                };
                send_command(
                    &mut write,
                    WebSocketMessage {
                        command: Command::Move,
                        args: serde_json::to_value(args2).unwrap(),
                    },
                )
                .await?;
            }
            Command::Move => todo!(),
            Command::EndMatch => {
                let response: EndMatchArgs = serde_json::from_value(message.args).unwrap();
                println!("Matched ended: {}, winner: {:?}.", response.reason, response.winner);
            },
        }
    }

    Ok(())
}
