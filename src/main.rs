mod args;

use std::net::{Ipv6Addr, TcpListener, TcpStream};

use clap::Parser;
use crossterm::{
    cursor,
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tui_textarea::{Input, Key, TextArea};

use args::*;

fn main() {
    let args = MessageTuiArgs::parse();
    let mut username = "anonymous".to_owned();
    match args.subcommand {
        MessageTuiSubcommand::Listen(ListenCommand { name, port }) => {
            println!("Listening on port {} as {}...", port, name);

            username = name;

            // Open a socket and listen for incoming connections
            let listener = TcpListener::bind((Ipv6Addr::LOCALHOST, port)).unwrap();

            // Wait for a connection to be established
            let (stream, address) = listener.accept().unwrap();

            println!("Connection established with {}", address);
        }
        MessageTuiSubcommand::Connect(ConnectCommand {
            name,
            address,
            port,
        }) => {
            println!("Connecting to {} on port {} as {}...", address, port, name);

            username = name;

            // Open a socket and connect to the specified address
            let Ok(stream) = TcpStream::connect((address, port)) else {
                println!("Failed to connect to {}", address);
                return;
            };

            println!("Connection established with {}", address);
        }
    }

    let mut app = MessageApp::open(username);
    app.start_ui();

    app.close();
}

#[derive(Serialize, Deserialize)]
struct Message {
    sender: String,
    content: String,
}

impl Message {
    fn new(sender: &str, content: &str) -> Self {
        let (sender, content) = (sender.to_owned(), content.to_owned());
        Self { sender, content }
    }

    fn format(&self) -> Text<'_> {
        let sender = Span::styled(
            self.sender.as_str(),
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

        Text::from(Line::from(vec![
            sender,
            Span::styled(" > ", Style::default().fg(Color::LightGreen)),
            Span::from(self.content.as_str()),
        ]))
    }
}

struct MessageApp<'a> {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    username: String,
    messages: Vec<Message>,
    entry: TextArea<'a>,
}

impl<'a> MessageApp<'a> {
    fn open(username: String) -> Self {
        let mut stdout = std::io::stdout();
        enable_raw_mode().unwrap();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        Self {
            terminal,
            username,
            messages: Vec::new(),
            entry: TextArea::default(),
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

    fn start_ui(&mut self) {
        loop {
            self.render();
            match crossterm::event::read().unwrap().into() {
                Input { key: Key::Esc, .. } => break,
                Input {
                    key: Key::Enter, ..
                } => {
                    let content = self.entry.lines().to_owned().join("\n");
                    self.messages.push(Message::new(&self.username, &content));
                    self.entry = TextArea::default();
                }
                input => {
                    self.entry.input(input);
                }
            }
        }
    }

    fn render(&mut self) {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                .split(f.size());

            let messages = self
                .messages
                .iter()
                .map(|message| ListItem::new(message.format()))
                .collect::<Vec<_>>();

            let messages = List::new(messages).block(Block::default().borders(Borders::ALL));
            self.entry
                .set_block(Block::default().borders(Borders::ALL).title(Span::styled(
                    "Message",
                    Style::default().fg(Color::LightGreen),
                )));

            f.render_widget(messages, chunks[0]);
            f.render_widget(self.entry.widget(), chunks[1]);
        });
    }
}
