use clap::{Parser, Subcommand};
use anyhow::Result;
use std::fs;
use sha1::{Sha1, Digest};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;


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
    HashObject {
        file: std::path::PathBuf,
    }
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
        Command::HashObject { file } => {
            let content = std::fs::read(&file)?;
            let header = format!("blob {}\0", content.len());
            let mut store = header.into_bytes();
            store.extend(content);
           
            let mut hasher = Sha1::new();
            hasher.update(&store);
            let hash_result = hasher.finalize();
            let hash_hex = hex::encode(hash_result);
            
            let mut encoder= ZlibEncoder::new(Vec::new(), Compression::default());
            encoder.write_all(&store)?;
            let compressed_bytes = encoder.finish()?;
            
            let directory_name = &hash_hex[..2];
            let filename = &hash_hex[2..];
            
            fs::create_dir_all(format!(".mimir/objects/{}", directory_name))?;
            fs::write(format!(".mimir/objects/{}/{}", directory_name,filename), compressed_bytes)?;
            
        }
    }
    
    Ok(())    
}