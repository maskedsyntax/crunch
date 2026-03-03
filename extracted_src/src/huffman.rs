use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;
use anyhow::{Result, anyhow};
use crate::bit_io::{BitReader, BitWriter};
use std::io::{Read, Write};

#[derive(Debug, PartialEq, Eq)]
pub enum HuffmanNode {
    Internal {
        left: Box<HuffmanNode>,
        right: Box<HuffmanNode>,
    },
    Leaf {
        symbol: u8,
    },
}

pub struct Huffman {
    codes: HashMap<u8, (u64, u8)>, // symbol -> (code, bits_count)
    tree: Option<HuffmanNode>,
}

impl Huffman {
    pub fn from_frequencies(frequencies: HashMap<u8, u64>) -> Self {
        if frequencies.is_empty() {
            return Self { codes: HashMap::new(), tree: None };
        }

        #[derive(Eq, PartialEq)]
        struct NodeWrapper(HuffmanNode, u64);
        impl Ord for NodeWrapper {
            fn cmp(&self, other: &Self) -> Ordering {
                other.1.cmp(&self.1) // Min-heap
            }
        }
        impl PartialOrd for NodeWrapper {
            fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
                Some(self.cmp(other))
            }
        }

        let mut heap = BinaryHeap::new();
        for (symbol, weight) in frequencies {
            heap.push(NodeWrapper(HuffmanNode::Leaf { symbol }, weight));
        }

        while heap.len() > 1 {
            let NodeWrapper(left, w1) = heap.pop().unwrap();
            let NodeWrapper(right, w2) = heap.pop().unwrap();
            heap.push(NodeWrapper(
                HuffmanNode::Internal {
                    left: Box::new(left),
                    right: Box::new(right),
                },
                w1 + w2,
            ));
        }

        let NodeWrapper(root, _) = heap.pop().unwrap();
        let mut codes = HashMap::new();
        Self::generate_codes(&root, 0, 0, &mut codes);

        Self {
            codes,
            tree: Some(root),
        }
    }

    fn generate_codes(node: &HuffmanNode, current_code: u64, bits_count: u8, codes: &mut HashMap<u8, (u64, u8)>) {
        match node {
            HuffmanNode::Leaf { symbol } => {
                let count = if bits_count == 0 { 1 } else { bits_count };
                codes.insert(*symbol, (current_code, count));
            }
            HuffmanNode::Internal { left, right } => {
                Self::generate_codes(left, current_code << 1, bits_count + 1, codes);
                Self::generate_codes(right, (current_code << 1) | 1, bits_count + 1, codes);
            }
        }
    }

    pub fn get_code_lengths(&self) -> HashMap<u8, u8> {
        let mut lengths = HashMap::new();
        for (symbol, (_, bits)) in &self.codes {
            lengths.insert(*symbol, *bits);
        }
        lengths
    }

    pub fn from_code_lengths(lengths: HashMap<u8, u8>) -> Self {
        if lengths.is_empty() {
            return Self { codes: HashMap::new(), tree: None };
        }

        let mut sorted_symbols: Vec<(u8, u8)> = lengths.into_iter().collect();
        // Sort by length, then symbol
        sorted_symbols.sort_by(|a, b| a.1.cmp(&b.1).then(a.0.cmp(&b.0)));

        let mut codes = HashMap::new();
        let mut current_code: u64 = 0;
        let mut last_len = sorted_symbols[0].1;

        for (symbol, len) in sorted_symbols {
            if len > last_len {
                current_code <<= len - last_len;
            }
            codes.insert(symbol, (current_code, len));
            current_code += 1;
            last_len = len;
        }

        let mut root = HuffmanNode::Internal {
            left: Box::new(HuffmanNode::Leaf { symbol: 0 }),
            right: Box::new(HuffmanNode::Leaf { symbol: 0 }),
        };

        for (symbol, (code, len)) in &codes {
            let mut current = &mut root;
            for i in (0..*len).rev() {
                let bit = (code >> i) & 1 == 1;
                if i > 0 {
                    if let HuffmanNode::Internal { left, right } = current {
                        let target = if bit { right.as_mut() } else { left.as_mut() };
                        // Ensure target is internal
                        if let HuffmanNode::Leaf { .. } = target {
                            *target = HuffmanNode::Internal {
                                left: Box::new(HuffmanNode::Leaf { symbol: 0 }),
                                right: Box::new(HuffmanNode::Leaf { symbol: 0 }),
                            };
                        }
                        current = target;
                    }
                } else {
                    if let HuffmanNode::Internal { left, right } = current {
                        if bit {
                            *right = Box::new(HuffmanNode::Leaf { symbol: *symbol });
                        } else {
                            *left = Box::new(HuffmanNode::Leaf { symbol: *symbol });
                        }
                    }
                }
            }
        }

        Self {
            codes,
            tree: Some(root),
        }
    }

    pub fn encode<R: Read, W: Write>(&self, mut input: R, output: W) -> Result<()> {
        let mut writer = BitWriter::new(output);
        let mut buf = [0u8; 1024];
        loop {
            let n = input.read(&mut buf)?;
            if n == 0 { break; }
            for &byte in &buf[..n] {
                let (code, bits) = self.codes.get(&byte)
                    .ok_or_else(|| anyhow!("Symbol not found in Huffman codes: {}", byte))?;
                writer.write_bits(*code, *bits)?;
            }
        }
        writer.flush_bits()?;
        Ok(())
    }

    pub fn decode<R: Read, W: Write>(&self, input: R, mut output: W, mut original_size: u64) -> Result<()> {
        let mut reader = BitReader::new(input);
        let root = self.tree.as_ref().ok_or_else(|| anyhow!("Huffman tree is empty"))?;

        while original_size > 0 {
            let mut current = root;
            while let HuffmanNode::Internal { left, right } = current {
                let bit = reader.read_bit()?.ok_or_else(|| anyhow!("Unexpected EOF during decoding"))?;
                current = if bit { right } else { left };
            }
            if let HuffmanNode::Leaf { symbol } = current {
                output.write_all(&[*symbol])?;
                original_size -= 1;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_huffman_roundtrip() -> Result<()> {
        let data = b"hello huffman";
        let mut frequencies = HashMap::new();
        for &byte in data {
            *frequencies.entry(byte).or_insert(0) += 1;
        }

        let huffman = Huffman::from_frequencies(frequencies);
        let mut compressed = Vec::new();
        huffman.encode(Cursor::new(data), &mut compressed)?;

        let mut decompressed = Vec::new();
        huffman.decode(Cursor::new(compressed), &mut decompressed, data.len() as u64)?;

        assert_eq!(decompressed, data);
        Ok(())
    }

    #[test]
    fn test_canonical_huffman_roundtrip() -> Result<()> {
        let data = b"the quick brown fox jumps over the lazy dog";
        let mut frequencies = HashMap::new();
        for &byte in data {
            *frequencies.entry(byte).or_insert(0) += 1;
        }

        let h1 = Huffman::from_frequencies(frequencies);
        let lengths = h1.get_code_lengths();
        
        let h2 = Huffman::from_code_lengths(lengths);
        let mut compressed = Vec::new();
        h2.encode(Cursor::new(data), &mut compressed)?;

        let mut decompressed = Vec::new();
        h2.decode(Cursor::new(compressed), &mut decompressed, data.len() as u64)?;

        assert_eq!(decompressed, data);
        Ok(())
    }
}
