mod influx_db;
#[allow(dead_code)]
mod util;

use anyhow::Result;
use kankyo;
use std::io;
use termion::event::Key;
use termion::input::MouseTerminal;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Gauge, Widget};
use tui::Terminal;

use crate::influx_db::{InfluxDBConnection, StorageUsage};
use crate::util::event::{Event, Events};

struct App {
    storage: Vec<StorageUsage>,
    pub connection: InfluxDBConnection,
}

impl App {
    fn new() -> Result<App> {
        let connection = InfluxDBConnection::new()?;
        Ok(App {
            storage: Vec::new(),
            connection,
        })
    }

    async fn update(&mut self) {
        let response = self.connection.get_storage_load().await;
        match response {
            Ok(data) => {
                self.storage = data;
            }
            Err(e) => {
                println!("{:?}", e);
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    kankyo::init()?;

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let events = Events::new();

    // // App
    let mut app = App::new()?;

    loop {
        terminal.draw(|mut f| {
            let number_of_storage_devices = app.storage.len();
            let mut constraints: Vec<Constraint> = Vec::with_capacity(number_of_storage_devices);
            for _ in 0..number_of_storage_devices {
                constraints.push(Constraint::Percentage(
                    (100 / number_of_storage_devices) as u16,
                ));
            }

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(constraints.as_ref())
                .split(f.size());

            for (index, drive) in app.storage.iter().enumerate() {
                let colour: Color;

                if drive.value > 0.0 && drive.value < 60.0 {
                    colour = Color::LightGreen;
                } else if drive.value >= 60.0 && drive.value < 85.0 {
                    colour = Color::LightYellow;
                } else {
                    colour = Color::LightRed;
                }

                Gauge::default()
                    .block(
                        Block::default()
                            .title(&drive.hardware)
                            .borders(Borders::ALL),
                    )
                    .style(Style::default().fg(colour))
                    .percent(drive.value as u16)
                    .render(&mut f, chunks[index]);
            }
        })?;

        match events.next()? {
            Event::Input(input) => {
                if input == Key::Char('q') {
                    break;
                }
            }
            Event::Tick => {
                app.update().await;
            }
        }
    }

    Ok(())
}
