# Crunch: Custom File Compressor & Archiver - Development Plan

## 1. Project Initialization
- [x] Initialize Cargo project.
- [x] Set up basic CLI structure with `clap`.
- [x] Integrate error handling with `anyhow`.

## 2. Bit-level I/O Infrastructure
- [x] Implement `BitWriter` for writing individual bits to a stream.
- [x] Implement `BitReader` for reading individual bits from a stream.
- [x] Unit tests for bit-level operations.

## 3. Huffman Coding (Lossless Compression)
- [x] Frequency analysis of input data.
- [x] Huffman tree construction using a Priority Queue.
- [x] Code generation from the Huffman tree.
- [x] Encoding logic (Input -> Huffman Codes -> Bits).
- [x] Decoding logic (Bits -> Huffman Tree -> Original Data).
- [x] Canonical Huffman coding for efficient tree storage in the archive.

## 4. Archive Format & Metadata
- [x] Define the `crunch` file format (Header, Metadata, Compressed Data).
- [x] Store checksums (CRC32) for integrity verification.
- [x] Support for multiple compression algorithms (selectable via CLI).

## 5. Multi-File Archiving
- [x] Recursive directory traversal.
- [x] Metadata storage for file names, sizes, and relative paths.
- [x] Archiving multiple files into a single `.crunch` file.

## 6. LZ77 (Dictionary-Based Compression)
- [x] Sliding window match finding.
- [x] Reference vs. Literal encoding.
- [x] Integration into the archiver.

## 7. Progress & Visualization
- [x] Real-time progress bars using `indicatif`.
- [x] Terminal-based frequency visualization (histograms).

## 8. Verification & Benchmarking
- [x] Automatic checksum verification during decompression.
- [x] Benchmarking utility to compare against standard ZIP implementations.
