use bitdefender::tui::App;

const SERVER_URL: &str = "wss://bitdefenders.cvjd.me/ws";
const MY_NAME: &str = "Balcus";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();
    let result = App::new(MY_NAME, SERVER_URL).run(&mut terminal).await;
    ratatui::restore();
    result
}
