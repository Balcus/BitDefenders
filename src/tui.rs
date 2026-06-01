use crate::{
    ai::{decide_actions, grid::Grid},
    protocol::{Command, WebSocketMessage, send_command},
    types::{
        self, Action, ChallengeArgs, EndMatchArgs, EnemySide, Hero, LoginArgs, StartMatchArgs,
        StartTurnArgs,
    },
};
use anyhow::Context;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind};
use futures::{SinkExt, StreamExt, stream::SplitSink};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, List, ListItem, ListState, Paragraph},
};
use std::fmt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, connect_async, tungstenite::Message};

#[derive(Debug, Clone, PartialEq)]
pub enum MenuOption {
    Practice,
    Challenge,
    Ranked,
}

impl fmt::Display for MenuOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuOption::Practice => write!(f, "  Practice"),
            MenuOption::Challenge => write!(f, "  Challenge"),
            MenuOption::Ranked => write!(f, "  Ranked"),
        }
    }
}

#[derive(Debug, PartialEq)]
enum AppState {
    Menu,
    WaitingForMatch,
    InMatch,
    MatchEnded { reason: String, winner: Option<i32> },
}

pub struct App {
    state: AppState,
    menu_options: Vec<MenuOption>,
    menu_state: ListState,
    exit: bool,
    url: String,
    player_name: String,
    player_id: i32,
    write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,
    match_config: Option<types::GameConfig>,
    enemy_side: Option<EnemySide>,
    grid: Grid,
    log: Vec<String>,
}

impl App {
    pub fn new(player_name: impl Into<String>, server_url: impl Into<String>) -> Self {
        let mut menu_state = ListState::default();
        menu_state.select(Some(0));
        Self {
            state: AppState::Menu,
            menu_options: vec![
                MenuOption::Practice,
                MenuOption::Challenge,
                MenuOption::Ranked,
            ],
            menu_state,
            exit: false,
            url: server_url.into(),
            player_name: player_name.into(),
            player_id: 0,
            write: None,
            match_config: None,
            enemy_side: None,
            grid: Grid::default(),
            log: Vec::new(),
        }
    }

    fn log(&mut self, msg: impl Into<String>) {
        let s = msg.into();
        if self.log.len() >= 8 {
            self.log.remove(0);
        }
        self.log.push(s);
    }

    fn selected_option(&self) -> &MenuOption {
        let i = self.menu_state.selected().unwrap_or(0);
        &self.menu_options[i]
    }

    fn menu_up(&mut self) {
        let i = self.menu_state.selected().unwrap_or(0);
        let next = if i == 0 {
            self.menu_options.len() - 1
        } else {
            i - 1
        };
        self.menu_state.select(Some(next));
    }

    fn menu_down(&mut self) {
        let i = self.menu_state.selected().unwrap_or(0);
        let next = (i + 1) % self.menu_options.len();
        self.menu_state.select(Some(next));
    }

    pub async fn run(mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        self.log(format!("Connecting to {} …", self.url));
        let (ws, _) = connect_async(&self.url)
            .await
            .context("WebSocket connect")?;
        let (write, mut read) = ws.split();
        self.write = Some(write);
        self.log("Connected. Waiting for Hello…");

        let (key_tx, mut key_rx) = mpsc::unbounded_channel::<KeyEvent>();
        std::thread::spawn(move || {
            loop {
                if let Ok(Event::Key(key)) = crossterm::event::read() {
                    if key_tx.send(key).is_err() {
                        break;
                    }
                }
            }
        });

        loop {
            if self.exit {
                break;
            }

            terminal.draw(|frame| self.draw(frame))?;

            tokio::select! {
                ws_msg = read.next() => {
                    match ws_msg {
                        Some(Ok(msg)) => {
                            if let Err(e) = self.handle_ws_message(msg).await {
                                self.log(format!("WS error: {e}"));
                            }
                        }
                        Some(Err(e)) => {
                            self.log(format!("WebSocket error: {e}"));
                            break;
                        }
                        None => {
                            self.log("Server disconnected.");
                            break;
                        }
                    }
                }

                Some(key) = key_rx.recv() => {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key).await?;
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_ws_message(&mut self, msg: Message) -> anyhow::Result<()> {
        match msg {
            Message::Ping(p) => {
                if let Some(w) = self.write.as_mut() {
                    w.send(Message::Pong(p)).await?;
                }
            }
            Message::Close(f) => {
                self.log(format!("Server closed connection: {f:?}"));
                self.exit = true;
            }
            Message::Text(text) => {
                let envelope: WebSocketMessage =
                    serde_json::from_str(&text).context("deserialize WS message")?;
                self.handle_command(envelope).await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn handle_command(&mut self, msg: WebSocketMessage) -> anyhow::Result<()> {
        match msg.command {
            Command::Hello => {
                self.log("HELLO  →  sending LOGIN");
                let w = self.write.as_mut().context("no write half")?;
                send_command(
                    w,
                    WebSocketMessage::with_args(
                        Command::Login,
                        &LoginArgs {
                            name: self.player_name.clone(),
                            version: types::PROTOCOL_VERSION,
                        },
                    )?,
                )
                .await?;
            }

            Command::Ready => {
                self.log("READY  — menu active, choose a mode and press Enter");
                self.state = AppState::Menu;
            }

            Command::StartMatch => {
                let args: StartMatchArgs =
                    serde_json::from_value(msg.args).context("parse START_MATCH")?;

                self.player_id = args.your_player_id;
                self.grid = Grid::from(args.config.width, args.config.height, &args.state.walls);

                let my_heroes: Vec<&Hero> = args
                    .state
                    .heroes
                    .iter()
                    .filter(|h| h.owner_id == self.player_id)
                    .collect();

                self.enemy_side =
                    if !my_heroes.is_empty() && my_heroes[0].y < args.config.height / 2 {
                        Some(EnemySide::Bottom)
                    } else {
                        Some(EnemySide::Top)
                    };

                self.match_config = Some(args.config);
                self.state = AppState::InMatch;
                self.log(format!(
                    "START_MATCH  you=player{}  map={}×{}",
                    self.player_id,
                    self.match_config.as_ref().unwrap().width,
                    self.match_config.as_ref().unwrap().height,
                ));
            }

            Command::StartTurn => {
                let args: StartTurnArgs =
                    serde_json::from_value(msg.args).context("parse START_TURN")?;
                self.log(format!("START_TURN  turn={}", args.turn));

                let config = self.match_config.as_ref().context("no match config")?;
                let actions = decide_actions(
                    self.player_id,
                    config,
                    &args.state,
                    args.turn,
                    self.enemy_side.clone(),
                    &mut self.grid,
                );

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

                if let Some(w) = self.write.as_mut() {
                    w.send_all(&mut futures::stream::iter(messages).map(Ok))
                        .await?;
                }
            }

            Command::EndMatch => {
                let args: EndMatchArgs =
                    serde_json::from_value(msg.args).context("parse END_MATCH")?;
                self.log(format!(
                    "END_MATCH  reason={}  winner={:?}",
                    args.reason, args.winner
                ));
                self.match_config = None;
                self.state = AppState::MatchEnded {
                    reason: args.reason,
                    winner: args.winner.and_then(|s| s.parse::<i32>().ok()),
                };
            }

            Command::Error => {
                let args: types::ErrorArgs =
                    serde_json::from_value(msg.args).context("parse ERROR")?;
                self.log(format!(
                    "ERROR [{}] {} (fatal={})",
                    args.code, args.message, args.fatal
                ));
                if args.fatal {
                    self.exit = true;
                }
            }

            other => {
                self.log(format!("unexpected command: {other:?}"));
            }
        }
        Ok(())
    }

    async fn handle_key(&mut self, key: KeyEvent) -> anyhow::Result<()> {
        match &self.state {
            AppState::Menu | AppState::MatchEnded { .. } => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.exit = true;
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.state == AppState::Menu {
                        self.menu_up();
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.state == AppState::Menu {
                        self.menu_down();
                    }
                }
                KeyCode::Enter => {
                    if matches!(self.state, AppState::MatchEnded { .. }) {
                        self.state = AppState::Menu;
                    } else {
                        self.start_selected_mode().await?;
                    }
                }
                _ => {}
            },
            AppState::WaitingForMatch | AppState::InMatch => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q') => {
                    self.exit = true;
                }
                _ => {}
            },
        }
        Ok(())
    }

    async fn start_selected_mode(&mut self) -> anyhow::Result<()> {
        let option = self.selected_option().clone();
        self.log(format!("→ Starting {:?}…", option));
        self.state = AppState::WaitingForMatch;

        match option {
            MenuOption::Practice => {
                let w = self.write.as_mut().context("no connection")?;
                send_command(w, WebSocketMessage::empty(Command::Practice)).await?;
            }
            MenuOption::Challenge => {
                let ch_args = ChallengeArgs {
                    ranked: Some(false),
                    name: None,
                    seed: None,
                };
                let w = self.write.as_mut().context("no connection")?;
                send_command(
                    w,
                    WebSocketMessage::with_args(Command::Challenge, &ch_args)?,
                )
                .await?;
            }
            MenuOption::Ranked => {
                let ch_args = ChallengeArgs {
                    ranked: Some(true),
                    name: None,
                    seed: None,
                };
                let w = self.write.as_mut().context("no connection")?;
                send_command(
                    w,
                    WebSocketMessage::with_args(Command::Challenge, &ch_args)?,
                )
                .await?;
            }
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(10)])
            .split(area);

        match &self.state {
            AppState::Menu => self.draw_menu(frame, chunks[0]),
            AppState::WaitingForMatch => self.draw_waiting(frame, chunks[0]),
            AppState::InMatch => self.draw_in_match(frame, chunks[0]),
            AppState::MatchEnded { reason, .. } => {
                let reason = reason.clone();
                self.draw_match_ended(frame, chunks[0], &reason);
            }
        }

        self.draw_log(frame, chunks[1]);
    }

    fn draw_menu(&mut self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        frame.render_widget(Paragraph::new("bitdefenders").centered(), chunks[0]);

        let items: Vec<ListItem> = self
            .menu_options
            .iter()
            .map(|opt| ListItem::new(Line::from(opt.to_string())))
            .collect();

        let list = List::new(items)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("* ");

        frame.render_stateful_widget(list, chunks[1], &mut self.menu_state);

        frame.render_widget(
            Paragraph::new("j/k  enter  q")
                .centered()
                .style(Style::default().fg(Color::DarkGray)),
            chunks[2],
        );
    }

    fn draw_waiting(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        frame.render_widget(Paragraph::new("waiting for match...").centered(), area);
    }

    fn draw_in_match(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let config_text = self
            .match_config
            .as_ref()
            .map_or_else(|| "?".to_string(), |c| format!("{}x{}", c.width, c.height));
        let enemy = format!(
            "{:?}",
            self.enemy_side.as_ref().unwrap_or(&EnemySide::Bottom)
        );
        let text = Text::from(vec![
            Line::from(format!(
                "player {}  map {}  enemy {}",
                self.player_id, config_text, enemy
            )),
            Line::from(""),
            Line::from(Span::styled(
                "playing...",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(Paragraph::new(text).centered(), area);
    }

    fn draw_match_ended(&self, frame: &mut Frame, area: ratatui::layout::Rect, reason: &str) {
        let text = Text::from(vec![
            Line::from(Span::styled(
                reason.to_string(),
                Style::default().fg(Color::DarkGray),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "enter to continue",
                Style::default().fg(Color::DarkGray),
            )),
        ]);
        frame.render_widget(Paragraph::new(text).centered(), area);
    }

    fn draw_log(&self, frame: &mut Frame, area: ratatui::layout::Rect) {
        let lines: Vec<Line> = self
            .log
            .iter()
            .map(|s| {
                Line::from(Span::styled(
                    s.clone(),
                    Style::default().fg(Color::DarkGray),
                ))
            })
            .collect();
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .block(Block::default().borders(ratatui::widgets::Borders::TOP)),
            area,
        );
    }
}
