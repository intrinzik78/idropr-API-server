/// commnad line parser for flow control
/// 
use clap::Parser;

use crate::enums::PrimaryCommand;

#[derive(Debug,Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: PrimaryCommand // single command, no args
}