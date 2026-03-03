use anyhow::{Result, anyhow};
use crate::bit_io::{BitReader, BitWriter};
use std::io::{Read, Write};

pub struct LZ77 {
    window_size: usize,
    max_match_len: usize,
}

impl LZ77 {
    pub fn new(window_size: usize, max_match_len: usize) -> Self {
        Self {
            window_size,
            max_match_len,
        }
    }

    pub fn encode<R: Read, W: Write>(&self, mut input: R, output: W) -> Result<()> {
        let mut data = Vec::new();
        input.read_to_end(&mut data)?;

        let mut writer = BitWriter::new(output);
        let mut pos = 0;

        while pos < data.len() {
            let (offset, length) = self.find_best_match(&data, pos);

            if length >= 3 {
                // Write reference: [1 (flag), offset (12 bits), length (8 bits)]
                writer.write_bit(true)?;
                writer.write_bits(offset as u64, 12)?;
                writer.write_bits(length as u64, 8)?;
                pos += length;
            } else {
                // Write literal: [0 (flag), byte (8 bits)]
                writer.write_bit(false)?;
                writer.write_bits(data[pos] as u64, 8)?;
                pos += 1;
            }
        }
        writer.flush_bits()?;
        Ok(())
    }

    fn find_best_match(&self, data: &[u8], pos: usize) -> (usize, usize) {
        let mut best_offset = 0;
        let mut best_len = 0;

        let start = if pos > self.window_size { pos - self.window_size } else { 0 };

        for offset in (start..pos).rev() {
            let mut len = 0;
            while len < self.max_match_len && 
                  pos + len < data.len() && 
                  data[offset + len] == data[pos + len] {
                len += 1;
            }

            if len > best_len {
                best_len = len;
                best_offset = pos - offset;
                if len == self.max_match_len { break; }
            }
        }

        (best_offset, best_len)
    }

    pub fn decode<R: Read, W: Write>(&self, input: R, mut output: W, original_size: u64) -> Result<()> {
        let mut reader = BitReader::new(input);
        let mut decompressed = Vec::with_capacity(original_size as usize);

        while (decompressed.len() as u64) < original_size {
            match reader.read_bit()? {
                Some(true) => {
                    // Reference
                    let offset = reader.read_bits(12)?.ok_or_else(|| anyhow!("Failed to read offset"))? as usize;
                    let length = reader.read_bits(8)?.ok_or_else(|| anyhow!("Failed to read length"))? as usize;
                    
                    let start = decompressed.len() - offset;
                    for i in 0..length {
                        let byte = decompressed[start + i];
                        decompressed.push(byte);
                    }
                }
                Some(false) => {
                    // Literal
                    let byte = reader.read_bits(8)?.ok_or_else(|| anyhow!("Failed to read literal"))? as u8;
                    decompressed.push(byte);
                }
                None => break,
            }
        }

        output.write_all(&decompressed)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_lz77_roundtrip() -> Result<()> {
        let data = b"abcabcabcabcabc";
        let lz = LZ77::new(4096, 255);
        let mut compressed = Vec::new();
        lz.encode(Cursor::new(data), &mut compressed)?;

        let mut decompressed = Vec::new();
        lz.decode(Cursor::new(compressed), &mut decompressed, data.len() as u64)?;

        assert_eq!(decompressed, data);
        Ok(())
    }
}
