mod args;

use std::io::{Read, Write};
use std::net::{Ipv6Addr, TcpListener, TcpStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

use clap::Parser;
use crossterm::cursor;
use crossterm::{
    event::DisableMouseCapture,
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tui_textarea::{Input, Key, TextArea};

use args::*;

const POLL_RATE: Duration = Duration::from_millis(200);

fn main() {
    let args = MessageTuiArgs::parse();

    let mut tcp_connection;
    let username;
    match args.subcommand {
        MessageTuiSubcommand::Listen(ListenCommand { name, port }) => {
            println!("Listening on port {} as {}...", port, name);

            username = name;

            // Open a socket and listen for incoming connections
            let listener = TcpListener::bind((Ipv6Addr::LOCALHOST, port)).unwrap();

            // Wait for a connection to be established
            let (stream, address) = listener.accept().unwrap();
            tcp_connection = stream;

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
            tcp_connection = stream;

            println!("Connection established with {}", address);
        }
    }

    // Create a channel for sending messages to the UI thread and for sending keypresses to the UI thread
    let (key_and_network_sender, ui_receiver): (Sender<Signal>, Receiver<Signal>) = mpsc::channel();
    let network_sender = key_and_network_sender;
    let key_sender = Some(network_sender.clone());
    // Create a channel for sending messages to the network thread
    let (ui_sender, network_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    // Spawn a thread to handle network communication
    thread::spawn(move || {
        tcp_connection.set_nonblocking(true).unwrap();
        loop {
            // Send any messages received from the UI thread
            if let Ok(message) = network_receiver.try_recv() {
                let message: String = serde_json::to_string(&message).unwrap();
                tcp_connection.write_all(message.as_bytes()).unwrap();
            }

            // Read messages from the network
            let mut message_buffer = [0; 1024];
            if let Ok(bytes_read) = tcp_connection.read(&mut message_buffer) {
                if bytes_read != 0 {
                    // Send the message, if one was recieved, to be displayed by the UI
                    if let Ok(message) = serde_json::from_slice(&message_buffer[..bytes_read]) {
                        network_sender.send(Signal::Message(message)).unwrap();
                    }
                }
            };

            // Poll the network at a fixed rate to avoid needless CPU overhead
            thread::sleep(POLL_RATE)
        }
    });

    // Hande the UI in the main thread
    let mut app = MessageApp::open(username, ui_sender, key_sender, ui_receiver);
    app.run_ui();
}

enum Signal {
    Message(Message),
    KeyPress(Input),
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    sender: String,
    content: String,
}

struct MessageApp<'a> {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    message_sender: Sender<Message>,
    key_sender: Option<Sender<Signal>>,
    receiver: Receiver<Signal>,
    username: String,
    messages: Vec<Message>,
    entry: TextArea<'a>,
}

impl<'a> MessageApp<'a> {
    fn open(
        username: String,
        message_sender: Sender<Message>,
        key_sender: Option<Sender<Signal>>,
        receiver: Receiver<Signal>,
    ) -> Self {
        let mut stdout = std::io::stdout();
        enable_raw_mode().unwrap();
        execute!(stdout, EnterAlternateScreen, DisableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        Self {
            terminal,
            message_sender,
            key_sender,
            receiver,
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
            cursor::Show
        )
        .unwrap();
    }

    fn run_ui(&mut self) {
        let key_sender = self.key_sender.take().unwrap();
        thread::spawn(move || loop {
            // Catch user input and send it to the UI thread
            let input = crossterm::event::read().unwrap().into();
            key_sender.send(Signal::KeyPress(input)).unwrap();
        });

        loop {
            // Frame update
            self.render();

            // Handle input from the user and messages from the network
            if let Ok(signal) = self.receiver.recv() {
                match signal {
                    Signal::Message(message) => {
                        self.messages.push(message);
                    }
                    Signal::KeyPress(input) => {
                        self.handle_input(input);
                    }
                }
            }
        }
    }

    fn handle_input(&mut self, input: Input) {
        match input {
            Input { key: Key::Esc, .. } => {
                // Close the UI and exit the program
                self.close();
                std::process::exit(0);
            }
            Input {
                key: Key::Enter, ..
            } => {
                // If the message is not empty, add it to the display and send it over the network
                let content = self.entry.lines().to_owned().join("\n");
                if content.is_empty() {
                    return;
                }

                self.messages.push(Message::new(&self.username, &content));
                self.message_sender
                    .send(Message::new(&self.username, &content))
                    .unwrap();

                // Clear the entry box
                self.entry = TextArea::default();
            }
            input => {
                self.entry.input(input);
            }
        }
    }

    fn render(&mut self) {
        self.terminal
            .draw(|f| {
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
            })
            .unwrap();
    }
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
