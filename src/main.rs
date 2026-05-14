use clap::{Parser, Subcommand};
use anyhow::Result;
use std::fs;

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
            fs::create_dir_all(".mimir/objects")?;
            fs::create_dir_all(".mimir/refs")?;
            fs::write(".mimir/HEAD", "ref: refs/heads/main\n")?;
            println!("Initialized empty mimir repo");
        } 
    }
    
    Ok(())    
}