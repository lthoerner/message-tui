use std::net::Ipv6Addr;

use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct MessageTuiArgs {
    #[clap(subcommand)]
    /// The type of operation to perform
    pub subcommand: MessageTuiSubcommand,
}

#[derive(Debug, Subcommand, Clone)]
pub enum MessageTuiSubcommand {
    Listen(ListenCommand),
    Connect(ConnectCommand),
}

#[derive(Debug, Args, Clone)]
#[clap(about = "Listen for an incoming connection from another user")]
pub struct ListenCommand {
    /// The port to listen on
    pub port: u16,
}

#[derive(Debug, Args, Clone)]
#[clap(about = "Connect to another user")]
pub struct ConnectCommand {
    /// The address to connect to
    pub address: Ipv6Addr,
    /// The port to connect to
    pub port: u16,
}
