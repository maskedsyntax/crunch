mod bit_io;
mod huffman;
mod lz77;
mod archive;
mod benchmark;
mod stats;
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
    /// Benchmark against ZIP
    Bench {
        /// Input file to benchmark
        #[arg(short, long)]
        input: PathBuf,
    },
    /// Show file statistics (frequency analysis)
    Stats {
        /// Input file to analyze
        #[arg(short, long)]
        input: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Compress { input, output, algorithm } => {
            let out_path = output.unwrap_or_else(|| {
                let mut p = input.clone();
                p.set_extension("crunch");
                p
            });
            println!("Compressing {:?} using {} algorithm...", input, algorithm);
            archive::Archiver::compress_files(vec![input], out_path, &algorithm)?;
        }
        Commands::Decompress { input, output } => {
            println!("Decompressing {:?}...", input);
            let out_dir = output.unwrap_or_else(|| PathBuf::from("."));
            archive::Archiver::extract_files(input, out_dir)?;
        }
        Commands::Archive { input, output } => {
            println!("Creating archive {:?} from {:?}...", output, input);
            archive::Archiver::compress_files(input, output, "huffman")?;
        }
        Commands::List { input } => {
            println!("Listing contents of archive {:?}...", input);
            let mut file = std::fs::File::open(input)?;
            let header = archive::ArchiveHeader::read_from(&mut file)?;
            println!("{:<25} {:<15} {:<15} {:<10}", "Name", "Original", "Compressed", "Method");
            for meta in header.files {
                println!("{:<25} {:<15} {:<15} {:?}", meta.name, meta.original_size, meta.compressed_size, meta.compression_type);
            }
        }
        Commands::Bench { input } => {
            println!("Benchmarking {:?} against ZIP...", input);
            let res = benchmark::run_benchmark(input)?;
            println!("\nBenchmark Results for '{}':", res.name);
            println!("{:<20} {:<20} {:<20}", "Metric", "Crunch (Huffman)", "ZIP (Deflate)");
            println!("{:-<60}", "");
            println!("{:<20} {:<20} {:<20}", "Size (bytes)", res.crunch_size, res.zip_size);
            println!("{:<20} {:<20?} {:<20?}", "Time", res.crunch_time, res.zip_time);
            let crunch_ratio = (res.crunch_size as f64 / res.original_size as f64) * 100.0;
            let zip_ratio = (res.zip_size as f64 / res.original_size as f64) * 100.0;
            println!("{:<20} {:<20.2}% {:<20.2}%", "Ratio", crunch_ratio, zip_ratio);
        }
        Commands::Stats { input } => {
            println!("Frequency Analysis for {:?}:", input);
            stats::print_frequency_histogram(input)?;
        }
    }

    Ok(())
}
