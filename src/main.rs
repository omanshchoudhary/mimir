use clap::{Parser, Subcommand};
use anyhow::Result;
use std::fs;
use sha1::{Sha1, Digest};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use std::io::Write;
use flate2::read::ZlibDecoder;
use std::io::Read;

#[derive(Parser, Debug)]
#[command(version, about, long_about=None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init,
    HashObject {
        file: std::path::PathBuf,
    },
    CatFile {
        #[arg(short='p')]
        pretty: bool,
        
        object:String,
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
            println!("{}",hash_hex);
        }
        Command::CatFile { pretty, object } => {
            if !pretty {
                anyhow::bail!("For now, you must use the -p flag");                
            }
            let directory_name = &object[..2];
            let filename = &object[2..];
            
            let path = format!(".mimir/objects/{}/{}", directory_name,filename);
            let compressed_bytes = std::fs::read(&path)?;
            let mut decoder = ZlibDecoder::new(&compressed_bytes[..]);
            let mut decompressed_bytes = Vec::new();
            decoder.read_to_end(&mut decompressed_bytes)?;
            
            let null_byte_index = decompressed_bytes.iter().position(|&b| b==0).unwrap();
            let content = &decompressed_bytes[null_byte_index+1..];
            std::io::stdout().write_all(content)?;
            
            println!("{} {}", compressed_bytes.len(), path)
        }
    }
    
    Ok(())    
}