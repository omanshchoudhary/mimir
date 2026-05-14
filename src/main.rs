use clap::{Parser, Subcommand};
use anyhow::Result;


#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Initialize a new and empty repo
    Init,
}

fn main()->Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Command::Init => {
            println!("Initializing repository");
        } 
    }
    
    Ok(())    
}