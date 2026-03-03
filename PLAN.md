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
- [ ] Frequency analysis of input data.
- [ ] Huffman tree construction using a Priority Queue.
- [ ] Code generation from the Huffman tree.
- [ ] Encoding logic (Input -> Huffman Codes -> Bits).
- [ ] Decoding logic (Bits -> Huffman Tree -> Original Data).
- [ ] Canonical Huffman coding for efficient tree storage in the archive.

## 4. Archive Format & Metadata
- [ ] Define the `crunch` file format (Header, Metadata, Compressed Data).
- [ ] Store checksums (CRC32) for integrity verification.
- [ ] Support for multiple compression algorithms (selectable via CLI).

## 5. Multi-File Archiving
- [ ] Recursive directory traversal.
- [ ] Metadata storage for file names, sizes, and relative paths.
- [ ] Archiving multiple files into a single `.crunch` file.

## 6. LZ77 (Dictionary-Based Compression)
- [ ] Sliding window match finding.
- [ ] Triplets encoding (Offset, Length, Literal).
- [ ] Integration with Huffman (DEFLATE-lite) if time permits.

## 7. Progress & Visualization
- [ ] Real-time progress bars using `indicatif`.
- [ ] Terminal-based frequency visualization (histograms).

## 8. Verification & Benchmarking
- [ ] Automatic checksum verification during decompression.
- [ ] Benchmarking utility to compare against standard ZIP implementations.
