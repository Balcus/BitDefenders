use anyhow::Context;
use futures_util::SinkExt;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;

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
    Shoot,
    EndMatch,
    Challenge,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub command: Command,
    pub args: serde_json::Value,
}

impl WebSocketMessage {
    pub fn empty(command: Command) -> Self {
        Self {
            command,
            args: json!({}),
        }
    }

    pub fn with_args<T: Serialize>(command: Command, args: &T) -> anyhow::Result<Self> {
        Ok(Self {
            command,
            args: serde_json::to_value(args).context("serialize args")?,
        })
    }
}

pub async fn send_command<S>(write: &mut S, msg: WebSocketMessage) -> anyhow::Result<()>
where
    S: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin,
{
    let text = serde_json::to_string(&msg).context("serialize message")?;
    write
        .send(Message::Text(text.into()))
        .await
        .context("send message")?;
    Ok(())
}
