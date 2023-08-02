mod args;

use std::net::Ipv6Addr;

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};

use args::*;

fn main() {
    let args = MessageTuiArgs::parse();
    match args.subcommand {
        MessageTuiSubcommand::Listen(ListenCommand { port }) => {
            println!("Listening on port {}...", port);
        }
        MessageTuiSubcommand::Connect(ConnectCommand { address, port }) => {
            println!("Connecting to {} on port {}...", address, port);
        }
    }

    // let app = MessageApp::open();
}

struct Sender {
    address: Ipv6Addr,
    name: String,
}

struct Message {
    sender: Sender,
    content: String,
}

struct MessageApp {
    messages: Vec<Message>,
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl MessageApp {
    fn open() -> Self {
        let mut stdout = std::io::stdout();
        enable_raw_mode().unwrap();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        Self {
            messages: Vec::new(),
            terminal,
        }
    }

    fn close(&mut self) {
        disable_raw_mode().unwrap();
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )
        .unwrap();
    }
}
