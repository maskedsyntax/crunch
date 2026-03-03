mod bit_io;
mod huffman;
mod archive;
use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "crunch")]
#[command(about = "A custom file compressor and archiver", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compress a single file
    Compress {
        /// The file to compress
        #[arg(short, long)]
        input: PathBuf,

        /// The output compressed file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// The algorithm to use (huffman, lz77)
        #[arg(short, long, default_value = "huffman")]
        algorithm: String,
    },
    /// Decompress a file
    Decompress {
        /// The compressed file to decompress
        #[arg(short, long)]
        input: PathBuf,

        /// The output decompressed file or directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Create an archive from multiple files or a directory
    Archive {
        /// Input files or directory
        #[arg(short, long)]
        input: Vec<PathBuf>,

        /// The output archive name
        #[arg(short, long)]
        output: PathBuf,
    },
    /// List files in an archive
    List {
        /// The archive file to list
        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress { input, output, algorithm } => {
            println!("Compressing {:?} using {} algorithm...", input, algorithm);
            let out_path = output.unwrap_or_else(|| {
                let mut p = input.clone();
                p.set_extension("crunch");
                p
            });
            archive::Archiver::compress_files(vec![input], out_path)?;
        }
        Commands::Decompress { input, output } => {
            println!("Decompressing {:?}...", input);
            let out_dir = output.unwrap_or_else(|| PathBuf::from("."));
            archive::Archiver::extract_files(input, out_dir)?;
        }
        Commands::Archive { input, output } => {
            println!("Creating archive {:?} from {:?}...", output, input);
            archive::Archiver::compress_files(input, output)?;
        }
        Commands::List { input } => {
            println!("Listing contents of archive {:?}...", input);
            let mut file = std::fs::File::open(input)?;
            let header = archive::ArchiveHeader::read_from(&mut file)?;
            println!("{:<20} {:<15} {:<15} {:<10}", "Name", "Original Size", "Compressed", "Method");
            for meta in header.files {
                println!("{:<20} {:<15} {:<15} {:?}", meta.name, meta.original_size, meta.compressed_size, meta.compression_type);
            }
        }
    }

    Ok(())
}
