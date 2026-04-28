use anyhow::Context;
use bitdefender::{
    play::{Action, decide_actions},
    protocol::{Command, WebSocketMessage, send_command},
    types::{self, EndMatchArgs, EnemySide, Hero, LoginArgs, StartMatchArgs, StartTurnArgs},
};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::Message};

const SERVER_URL: &str = "wss://bitdefenders.cvjd.me/ws";
const MY_NAME: &str = "Balcus";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (ws, _) = connect_async(SERVER_URL).await.context("connect")?;
    let (mut write, mut read) = ws.split();

    let mut player_id: i32 = 0;
    let mut match_config: Option<types::GameConfig> = None;

    let mut enemy_side: Option<EnemySide> = None;

    println!("Connected to {SERVER_URL}");

    while let Some(msg) = read.next().await {
        let msg = msg.context("read message")?;

        let text = match msg {
            Message::Text(t) => t,
            Message::Ping(payload) => {
                write.send(Message::Pong(payload)).await?;
                continue;
            }
            Message::Close(frame) => {
                println!("Connection closed: {frame:?}");
                break;
            }
            _ => continue,
        };

        let envelope: WebSocketMessage =
            serde_json::from_str(&text).context("deserialize message")?;

        match envelope.command {
            Command::Hello => {
                println!("HELLO");
                send_command(
                    &mut write,
                    WebSocketMessage::with_args(
                        Command::Login,
                        &LoginArgs {
                            name: MY_NAME.into(),
                            version: types::PROTOCOL_VERSION,
                        },
                    )?,
                )
                .await?;
            }

            Command::Ready => {
                println!("READY — requesting practice match");
                send_command(&mut write, WebSocketMessage::empty(Command::Practice)).await?;
            }

            Command::StartMatch => {
                let args: StartMatchArgs =
                    serde_json::from_value(envelope.args).context("parse START_MATCH")?;
                println!(
                    "START_MATCH  match_id={} you=player{}",
                    args.match_id, args.your_player_id
                );

                let my_heroes: Vec<&Hero> = args
                    .state
                    .heroes
                    .iter()
                    .filter(|h| h.owner_id == player_id)
                    .collect();
                enemy_side = if my_heroes[0].y < &args.config.height / 2 {
                    Some(EnemySide::Bottom)
                } else {
                    Some(EnemySide::Top)
                };

                player_id = args.your_player_id;
                match_config = Some(args.config);
            }

            Command::StartTurn => {
                let args: StartTurnArgs =
                    serde_json::from_value(envelope.args).context("parse START_TURN")?;
                println!("turn={}", args.turn);

                let config = match_config.as_ref().expect("config set by START_MATCH");
                let actions = decide_actions(
                    player_id,
                    config,
                    &args.state,
                    args.turn,
                    enemy_side.clone(),
                );

                // Construiești toate mesajele mai întâi
                let messages: Vec<Message> = actions
                    .into_iter()
                    .filter_map(|action| {
                        let ws_msg = match action {
                            Action::Move(a) => {
                                WebSocketMessage::with_args(Command::Move, &a).ok()?
                            }
                            Action::Shoot(a) => {
                                WebSocketMessage::with_args(Command::Shoot, &a).ok()?
                            }
                        };
                        Some(Message::Text(serde_json::to_string(&ws_msg).ok()?.into()))
                    })
                    .collect();

                // Trimiți toate deodată
                write
                    .send_all(&mut futures_util::stream::iter(messages).map(Ok))
                    .await?;
            }

            Command::EndMatch => {
                let args: EndMatchArgs =
                    serde_json::from_value(envelope.args).context("parse END_MATCH")?;
                println!("END_MATCH  reason={} winner={:?}", args.reason, args.winner);
                match_config = None;
            }

            Command::Error => {
                let args: types::ErrorArgs =
                    serde_json::from_value(envelope.args).context("parse ERROR")?;
                eprintln!(
                    "ERROR  [{}] {} (fatal={})",
                    args.code, args.message, args.fatal
                );
                if args.fatal {
                    break;
                }
            }

            Command::Login | Command::Practice | Command::Move | Command::Shoot => {
                eprintln!("Unexpected command from server: {:?}", envelope.command);
            }
        }
    }

    Ok(())
}
