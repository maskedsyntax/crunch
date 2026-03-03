mod bit_io;
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
            if let Some(out) = output {
                println!("Output will be saved to {:?}", out);
            }
        }
        Commands::Decompress { input, output } => {
            println!("Decompressing {:?}...", input);
            if let Some(out) = output {
                println!("Output will be saved to {:?}", out);
            }
        }
        Commands::Archive { input, output } => {
            println!("Creating archive {:?} from {:?}...", output, input);
        }
        Commands::List { input } => {
            println!("Listing contents of archive {:?}...", input);
        }
    }

    Ok(())
}
