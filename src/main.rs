mod args;

use std::io::{Read, Write};
use std::net::{Ipv6Addr, TcpListener, TcpStream};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc::{self, Receiver, Sender},
    Arc,
};
use std::thread;
use std::time::Duration;

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

const POLL_RATE: Duration = Duration::from_millis(500);

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

    // Create a channel for sending messages to the UI thread
    let (network_sender, ui_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();
    // Create a channel for sending messages to the network thread
    let (ui_sender, network_receiver): (Sender<Message>, Receiver<Message>) = mpsc::channel();

    // Whether the program should exit, indicating to the network thread to close the connection
    let should_exit = Arc::new(AtomicBool::new(false));
    let should_exit_network = should_exit.clone();

    let times_polled = Arc::new(AtomicUsize::new(0));
    let times_polled_network = times_polled.clone();

    // Spawn a thread to handle network communication
    thread::spawn(move || {
        tcp_connection.set_nonblocking(true).unwrap();
        loop {
            if should_exit_network.load(Ordering::Relaxed) {
                break;
            }

            times_polled_network.fetch_add(1, Ordering::Relaxed);

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
                        network_sender.send(message).unwrap();
                    }
                }
            };

            // Poll the network at a fixed rate to avoid needless CPU overhead
            thread::sleep(POLL_RATE)
        }
    });

    // Hande the UI in the main thread
    let mut app = MessageApp::open(username, ui_sender, ui_receiver);
    app.run_ui();

    // Close the network connection and UI once the user has pressed Escape
    should_exit.store(true, Ordering::Relaxed);
    app.close();

    // Print the number of times the network was polled
    println!("Polled {} times", times_polled.load(Ordering::Relaxed));
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    sender: String,
    content: String,
}

struct MessageApp<'a> {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
    sender: Sender<Message>,
    receiver: Receiver<Message>,
    username: String,
    messages: Vec<Message>,
    entry: TextArea<'a>,
}

impl<'a> MessageApp<'a> {
    fn open(username: String, sender: Sender<Message>, receiver: Receiver<Message>) -> Self {
        let mut stdout = std::io::stdout();
        enable_raw_mode().unwrap();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture).unwrap();

        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        Self {
            terminal,
            sender,
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
            DisableMouseCapture,
        )
        .unwrap();
    }

    fn run_ui(&mut self) {
        self.render();
        loop {
            match crossterm::event::read().unwrap().into() {
                Input { key: Key::Esc, .. } => break,
                Input {
                    key: Key::Enter, ..
                } => {
                    let content = self.entry.lines().to_owned().join("\n");
                    // Add the message to the display and send it over the network
                    self.messages.push(Message::new(&self.username, &content));
                    self.sender.send(Message::new(&self.username, &content));

                    // Clear the entry box
                    self.entry = TextArea::default();
                }
                input => {
                    self.entry.input(input);
                }
                // Don't update the UI for other events
                // ? Is this necessary? I think this is already a blocking call
                _ => continue,
            }

            if let Ok(message) = self.receiver.try_recv() {
                self.messages.push(message);
            }

            self.render();
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
