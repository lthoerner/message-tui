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
    /// The name you want to be identified by
    #[arg(default_value = "anonymous")]
    pub name: String,
    /// The address to listen for connections on
    #[arg(default_value_t = Ipv6Addr::LOCALHOST)]
    pub address: Ipv6Addr,
    /// The port to listen on
    #[arg(default_value = "12500")]
    pub port: u16,
    /// The rate at which to poll for incoming messages, in milliseconds.
    /// Higher values result in lower CPU usage, but lower values increase responsiveness
    #[arg(default_value = "100")]
    pub poll_rate: u16,
}

#[derive(Debug, Args, Clone)]
#[clap(about = "Connect to another user")]
pub struct ConnectCommand {
    /// The name you want to be identified by
    #[arg(default_value = "anonymous")]
    pub name: String,
    /// The address to connect to
    #[arg(default_value_t = Ipv6Addr::LOCALHOST)]
    pub address: Ipv6Addr,
    /// The port to connect to
    #[arg(default_value = "12500")]
    pub port: u16,
    /// The rate at which to poll for incom, in milliseconds.
    /// Higher values result in lower CPU usage, but lower values increase responsiveness
    #[arg(default_value = "100")]
    pub poll_rate: u16,
}
