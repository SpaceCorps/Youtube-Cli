//! Command dispatcher.

pub mod accounts;
pub mod login;
pub mod scrape;
pub mod search;

use crate::cli::Command;
use crate::error::Result;
use crate::readme;

pub fn run(command: Command) -> Result<()> {
    match command {
        Command::Search(args) => search::run(args),
        Command::Scrape(args) => scrape::run(args),
        Command::Login(args) => login::run(args),
        Command::Accounts(args) => accounts::run(args),
        Command::AgentReadme => {
            readme::print();
            Ok(())
        }
    }
}
