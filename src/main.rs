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

use args::*;

fn main() {
    let args = MessageTuiArgs::parse();
    match args.subcommand {
        MessageTuiSubcommand::Listen(ListenCommand { port }) => {
            println!("Listening on port {}...", port);

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

            // Open a socket and connect to the specified address
            let Ok(stream) = TcpStream::connect((address, port)) else {
                println!("Failed to connect to {}", address);
                return;
            };

            println!("Connection established with {}", address);
        }
    }

    let mut app = MessageApp::open();
    app.render();

    std::thread::sleep(std::time::Duration::from_secs(10));

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

struct MessageApp {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    messages: Vec<Message>,
    entry: String,
}

impl MessageApp {
    fn open() -> Self {
        let mut stdout = std::io::stdout();
        enable_raw_mode().unwrap();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        let messages = vec![
            Message::new("anonymous", "Hello, world!"),
            Message::new("anonymous", "How is everyone doing?"),
            Message::new("bill", "I'm doing well!"),
            Message::new("bill", "How about you?"),
            Message::new("anonymous", "I'm doing well too!"),
            Message::new("anonymous", "Thanks for asking!"),
        ];

        Self {
            terminal,
            messages,
            entry: String::new(),
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

    fn render(&mut self) {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                .split(f.size());

            let messages = self
                .messages
                .iter()
                .map(|message| ListItem::new(message.format()))
                .collect::<Vec<_>>();

            let messages = List::new(messages).block(Block::default().borders(Borders::ALL));
            let input_box = Paragraph::new(Line::from(vec![
                Span::styled("Enter message: ", Style::default().fg(Color::LightGreen)),
                Span::raw(self.entry.as_str()),
            ]))
            .block(Block::default().borders(Borders::ALL ^ Borders::TOP));

            f.render_widget(messages, chunks[0]);
            f.render_widget(input_box, chunks[1]);
        });
    }
}
