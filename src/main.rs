use anyhow::Result;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Read;
use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

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
    CommitTree {
        tree: String,

        #[arg(short = 'm')]
        message: String,

        #[arg(short = 'p')]
        parent: Option<String>,
    },

    Commit {
        #[arg(short = 'm')]
        message: String,
    },
    Log,
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
        Command::CommitTree {
            tree,
            message,
            parent,
        } => {
            let hash_hex = write_commit(&tree, &message, parent.as_deref())?;
            println!("{}", hash_hex);
        }
        Command::Commit { message } => {
            let head_content = fs::read_to_string(".mimir/HEAD")?;

            let ref_path = head_content.trim().strip_prefix("ref: ").unwrap();
            let branch_file_path = format!(".mimir/{}", ref_path);

            let parent_hash = if std::path::Path::new(&branch_file_path).exists() {
                Some(
                    std::fs::read_to_string(&branch_file_path)?
                        .trim()
                        .to_string(),
                )
            } else {
                None
            };

            let tree_hash = build_tree(std::path::Path::new("."))?;

            let commit_hash = write_commit(&tree_hash, &message, parent_hash.as_deref())?;

            if let Some(parent_dir) = std::path::Path::new(&branch_file_path).parent() {
                std::fs::create_dir_all(parent_dir)?;
            }
            std::fs::write(&branch_file_path, format!("{}\n", commit_hash))?;

            println!("Commited to branch main: {}", commit_hash);
        }
        Command::Log => {
            let head_content = std::fs::read_to_string(".mimir/HEAD")?;
            let ref_path = head_content.trim().strip_prefix("ref: ").unwrap();
            let branch_file_path = format!(".mimir/{}", ref_path);

            if !std::path::Path::new(&branch_file_path).exists() {
                println!("No commits yet!");
                return Ok(());
            }

            let mut current_hash = std::fs::read_to_string(&branch_file_path)?
                .trim()
                .to_string();

            loop {
                let directory_name = &current_hash[..2];
                let filename = &current_hash[2..];
                let path = format!(".mimir/objects/{}/{}", directory_name, filename);

                let compressed_bytes = std::fs::read(&path)?;
                let mut decoder = ZlibDecoder::new(&compressed_bytes[..]);
                let mut decompressed_bytes = Vec::new();
                decoder.read_to_end(&mut decompressed_bytes)?;

                let null_byte_index = decompressed_bytes.iter().position(|&b| b == 0).unwrap();
                let content_bytes = &decompressed_bytes[null_byte_index + 1..];

                let content_str = std::string::String::from_utf8_lossy(content_bytes);

                println!("commit {}", current_hash);

                let mut lines = content_str.lines();
                let mut parent_hash = None;

                for line in &mut lines {
                    if line.starts_with("parent ") {
                        parent_hash = Some(line.strip_prefix("parent ").unwrap().to_string());
                    } else if line.is_empty() {
                        break;
                    }
                }

                println!();
                for msg_line in lines {
                    println!("    {}", msg_line);
                }
                println!();

                if let Some(p) = parent_hash {
                    current_hash = p;
                } else {
                    break;
                }
            }
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

fn write_commit(tree: &str, message: &str, parent: Option<&str>) -> anyhow::Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let mut commit_content = String::new();
    commit_content.push_str(&format!("tree {}\n", tree));

    if let Some(parent_hash) = parent {
        commit_content.push_str(&format!("parent {}\n", parent_hash));
    }

    commit_content.push_str(&format!(
        "author Mimir User <mimir@example.com> {} +0000\n",
        timestamp
    ));
    commit_content.push_str(&format!(
        "committer Mimir User <mimir@example.com> {} +0000\n",
        timestamp
    ));
    commit_content.push_str("\n");
    commit_content.push_str(&format!("{}\n", message));

    let commit_bytes = commit_content.into_bytes();
    let header = format!("commit {}\0", commit_bytes.len());
    let mut store = header.into_bytes();
    store.extend(commit_bytes);

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
