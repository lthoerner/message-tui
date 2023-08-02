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
    #[arg(default_value = "anonymous")]
    /// The name you want to be identified by
    pub name: String,
    #[arg(default_value_t = Ipv6Addr::LOCALHOST)]
    /// The address to listen for connections on
    pub address: Ipv6Addr,
    #[arg(default_value = "12500")]
    /// The port to listen on
    pub port: u16,
}

#[derive(Debug, Args, Clone)]
#[clap(about = "Connect to another user")]
pub struct ConnectCommand {
    #[arg(default_value = "anonymous")]
    /// The name you want to be identified by
    pub name: String,
    #[arg(default_value_t = Ipv6Addr::LOCALHOST)]
    /// The address to connect to
    pub address: Ipv6Addr,
    #[arg(default_value = "12500")]
    /// The port to connect to
    pub port: u16,
}
