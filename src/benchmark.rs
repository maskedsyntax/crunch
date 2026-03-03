use std::path::Path;
use std::time::Instant;
use anyhow::Result;
use std::fs::File;
use std::io::{Read, Write};
use crate::archive::Archiver;
use zip::write::SimpleFileOptions;

pub struct BenchmarkResult {
    pub name: String,
    pub original_size: u64,
    pub crunch_size: u64,
    pub zip_size: u64,
    pub crunch_time: std::time::Duration,
    pub zip_time: std::time::Duration,
}

pub fn run_benchmark<P: AsRef<Path>>(input: P) -> Result<BenchmarkResult> {
    let path = input.as_ref();
    let mut file = File::open(path)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;
    let original_size = content.len() as u64;

    // Benchmark Crunch (Huffman)
    let start_crunch = Instant::now();
    let temp_crunch = std::env::temp_dir().join("crunch_bench.crunch");
    Archiver::compress_files(vec![path], &temp_crunch, "huffman")?;
    let crunch_time = start_crunch.elapsed();
    let crunch_size = std::fs::metadata(&temp_crunch)?.len();

    // Benchmark ZIP
    let start_zip = Instant::now();
    let temp_zip = std::env::temp_dir().join("crunch_bench.zip");
    let zip_file = File::create(&temp_zip)?;
    let mut zip = zip::ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    zip.start_file(path.file_name().unwrap().to_string_lossy(), options)?;
    zip.write_all(&content)?;
    zip.finish()?;
    let zip_time = start_zip.elapsed();
    let zip_size = std::fs::metadata(&temp_zip)?.len();

    // Clean up
    let _ = std::fs::remove_file(temp_crunch);
    let _ = std::fs::remove_file(temp_zip);

    Ok(BenchmarkResult {
        name: path.file_name().unwrap().to_string_lossy().to_string(),
        original_size,
        crunch_size,
        zip_size,
        crunch_time,
        zip_time,
    })
}
