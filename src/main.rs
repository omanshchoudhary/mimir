use anyhow::Result;
use clap::{Parser, Subcommand};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Read;
use std::io::Write;


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
        #[arg(short = 'p')]
        pretty: bool,

        object: String,
    },
    WriteTree,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init => {
            fs::create_dir_all(".mimir/objects")?;
            fs::create_dir_all(".mimir/refs")?;
            fs::write(".mimir/HEAD", "ref: refs/heads/main\n")?;
            println!("Initialized empty mimir repo");
        }
        Command::HashObject { file } => {
            let hash = write_blob(&file)?;
            println!("{}", hash);
        }
        Command::CatFile { pretty, object } => {
            if !pretty {
                anyhow::bail!("For now, you must use the -p flag");
            }
            let directory_name = &object[..2];
            let filename = &object[2..];

            let path = format!(".mimir/objects/{}/{}", directory_name, filename);
            let compressed_bytes = std::fs::read(&path)?;
            let mut decoder = ZlibDecoder::new(&compressed_bytes[..]);
            let mut decompressed_bytes = Vec::new();
            decoder.read_to_end(&mut decompressed_bytes)?;

            let null_byte_index = decompressed_bytes.iter().position(|&b| b == 0).unwrap();
            let content = &decompressed_bytes[null_byte_index + 1..];
            std::io::stdout().write_all(content)?;

            println!("{} {}", compressed_bytes.len(), path)
        }
        Command::WriteTree => {
            let hash = build_tree(std::path::Path::new("."))?;
            println!("{}", hash)
        }
    }

    Ok(())
}

fn write_blob(file: &std::path::Path) -> anyhow::Result<String> {
    let content = std::fs::read(&file)?;
    let header = format!("blob {}\0", content.len());
    let mut store = header.into_bytes();
    store.extend(content);

    let mut hasher = Sha1::new();
    hasher.update(&store);
    let hash_result = hasher.finalize();
    let hash_hex = hex::encode(hash_result);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&store)?;
    let compressed_bytes = encoder.finish()?;

    let directory_name = &hash_hex[..2];
    let filename = &hash_hex[2..];

    fs::create_dir_all(format!(".mimir/objects/{}", directory_name))?;
    fs::write(
        format!(".mimir/objects/{}/{}", directory_name, filename),
        compressed_bytes,
    )?;

    Ok(hash_hex)
}

fn build_tree(path: &std::path::Path) -> anyhow::Result<String> {
    let mut entries: Vec<_> = std::fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name() != ".git" && e.file_name() != ".mimir" && e.file_name() != "target"
        })
        .collect();
    entries.sort_by_key(|e| e.file_name());

    let mut tree_entries_bytes = Vec::new();

    for entry in entries {
        let file_type = entry.file_type()?;
        let file_name = entry.file_name().into_string().unwrap();
        let path = entry.path();

        let mode: &str;
        let hash_hex: String;

        if file_type.is_dir() {
            mode = "40000";
            hash_hex = build_tree(&path)?;
        } else {
            mode = "100644";
            hash_hex = write_blob(&path)?;
        }

        let entry_header = format!("{} {}\0", mode, file_name);
        let hash_bytes = hex::decode(hash_hex)?;
        tree_entries_bytes.extend(entry_header.into_bytes());
        tree_entries_bytes.extend(hash_bytes);
    }
    let tree_header = format!("tree {}\0", tree_entries_bytes.len());
    let mut tree = tree_header.into_bytes();
    tree.extend(tree_entries_bytes);

    let mut hasher = Sha1::new();
    hasher.update(&tree);
    let hash_result = hasher.finalize();
    let final_hash_hex = hex::encode(hash_result);

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&tree)?;
    let compressed_bytes = encoder.finish()?;

    let directory_name = &final_hash_hex[..2];
    let filename = &final_hash_hex[2..];

    fs::create_dir_all(format!(".mimir/objects/{}", directory_name))?;
    fs::write(
        format!(".mimir/objects/{}/{}", directory_name, filename),
        compressed_bytes,
    )?;
    Ok(final_hash_hex)
}
