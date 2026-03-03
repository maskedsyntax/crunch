use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::time::SystemTime;
use std::io::{Read, Write};
use std::fs::File;
use std::path::Path;
use anyhow::{Result, anyhow};
use crate::huffman::Huffman;
use crc32fast::Hasher;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CompressionType {
    Huffman,
    LZ77,
    None,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileMetadata {
    pub name: String,
    pub original_size: u64,
    pub compressed_size: u64,
    pub checksum: u32,
    pub compression_type: CompressionType,
    pub huffman_lengths: Option<HashMap<u8, u8>>,
    pub modified: Option<SystemTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArchiveHeader {
    pub magic: [u8; 6], // "CRUNCH"
    pub version: u32,
    pub files: Vec<FileMetadata>,
}

impl ArchiveHeader {
    pub const MAGIC: [u8; 6] = *b"CRUNCH";
    pub const VERSION: u32 = 1;

    pub fn new(files: Vec<FileMetadata>) -> Self {
        Self {
            magic: Self::MAGIC,
            version: Self::VERSION,
            files,
        }
    }

    pub fn write_to<W: Write>(&self, mut writer: W) -> Result<()> {
        let serialized = bincode::serialize(self)?;
        let len = serialized.len() as u64;
        writer.write_all(&len.to_le_bytes())?;
        writer.write_all(&serialized)?;
        Ok(())
    }

    pub fn read_from<R: Read>(mut reader: R) -> Result<Self> {
        let mut len_buf = [0u8; 8];
        reader.read_exact(&mut len_buf)?;
        let len = u64::from_le_bytes(len_buf);
        let mut buf = vec![0u8; len as usize];
        reader.read_exact(&mut buf)?;
        let header: ArchiveHeader = bincode::deserialize(&buf)?;
        if header.magic != Self::MAGIC {
            return Err(anyhow!("Invalid archive magic bytes"));
        }
        Ok(header)
    }
}

pub struct Archiver;

impl Archiver {
    pub fn compress_files<P: AsRef<Path>>(inputs: Vec<P>, output: P) -> Result<()> {
        let mut file_metas = Vec::new();
        let mut compressed_data = Vec::new();
        let mut all_files = Vec::new();

        for input in inputs {
            let path = input.as_ref();
            if path.is_file() {
                all_files.push((path.to_path_buf(), path.file_name().unwrap().to_string_lossy().to_string()));
            } else if path.is_dir() {
                Self::collect_files(path, path, &mut all_files)?;
            }
        }

        for (full_path, relative_name) in all_files {
            let mut file = File::open(&full_path)?;
            let mut content = Vec::new();
            file.read_to_end(&mut content)?;

            let original_size = content.len() as u64;
            let mut hasher = Hasher::new();
            hasher.update(&content);
            let checksum = hasher.finalize();

            // Calculate frequencies
            let mut frequencies = HashMap::new();
            for &byte in &content {
                *frequencies.entry(byte).or_insert(0) += 1;
            }

            let huffman_temp = Huffman::from_frequencies(frequencies);
            let lengths = huffman_temp.get_code_lengths();
            let huffman = Huffman::from_code_lengths(lengths.clone());

            let mut compressed_buf = Vec::new();
            huffman.encode(&content[..], &mut compressed_buf)?;

            let meta = FileMetadata {
                name: relative_name,
                original_size,
                compressed_size: compressed_buf.len() as u64,
                checksum,
                compression_type: CompressionType::Huffman,
                huffman_lengths: Some(lengths),
                modified: full_path.metadata()?.modified().ok(),
            };

            file_metas.push(meta);
            compressed_data.push(compressed_buf);
        }

        let header = ArchiveHeader::new(file_metas);
        let mut out_file = File::create(output)?;
        header.write_to(&mut out_file)?;

        for data in compressed_data {
            out_file.write_all(&data)?;
        }

        Ok(())
    }

    fn collect_files(root: &Path, current: &Path, files: &mut Vec<(std::path::PathBuf, String)>) -> Result<()> {
        for entry in std::fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                let relative_name = path.strip_prefix(root.parent().unwrap_or(root))?
                    .to_string_lossy()
                    .to_string();
                files.push((path, relative_name));
            } else if path.is_dir() {
                Self::collect_files(root, &path, files)?;
            }
        }
        Ok(())
    }

    pub fn extract_files<P: AsRef<Path>>(input: P, output_dir: P) -> Result<()> {
        let mut in_file = File::open(input)?;
        let header = ArchiveHeader::read_from(&mut in_file)?;

        let out_dir = output_dir.as_ref();
        if !out_dir.exists() {
            std::fs::create_dir_all(out_dir)?;
        }

        for meta in header.files {
            let mut compressed_buf = vec![0u8; meta.compressed_size as usize];
            in_file.read_exact(&mut compressed_buf)?;

            let out_path = out_dir.join(&meta.name);
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut out_file = File::create(out_path)?;

            match meta.compression_type {
                CompressionType::Huffman => {
                    let huffman = Huffman::from_code_lengths(meta.huffman_lengths.ok_or_else(|| anyhow!("Missing Huffman lengths"))?);
                    let mut decompressed_buf = Vec::new();
                    huffman.decode(&compressed_buf[..], &mut decompressed_buf, meta.original_size)?;
                    
                    // Verify checksum
                    let mut hasher = Hasher::new();
                    hasher.update(&decompressed_buf);
                    if hasher.finalize() != meta.checksum {
                        return Err(anyhow!("Checksum mismatch for file: {}", meta.name));
                    }

                    out_file.write_all(&decompressed_buf)?;
                }
                _ => return Err(anyhow!("Unsupported compression type")),
            }
        }

        Ok(())
    }
}
