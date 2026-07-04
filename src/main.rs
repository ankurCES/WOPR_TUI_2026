mod app;
mod config;
mod event;
mod game;
mod llm;
mod mode;
mod state;
mod terminal;
mod ui;
mod view;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    terminal::install_panic_hook();
    tokio::spawn(terminal::wait_for_signal());
    app::App::new().run().await
}
