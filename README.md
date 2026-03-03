# crunch

a tool to compress and archive files. i built this to get hands-on with huffman coding and lz77. it's a standalone archiver that handles multi-file structures and verifies data integrity.

## usage

you'll need `rust` and `cargo`.

### compress a file
`cargo run -- compress -i data.txt -a huffman`
(swap `huffman` for `lz77` if you want dictionary-based compression)

### decompress
`cargo run -- decompress -i data.crunch -o ./extracted`

### archive a directory
`cargo run -- archive -i src/ -o backup.crunch`

### see what's inside
`cargo run -- list -i backup.crunch`

### frequency analysis
`cargo run -- stats -i data.txt`
shows a histogram of byte frequencies in the terminal.

### benchmark
`cargo run -- bench -i data.txt`
compares crunch (huffman) against the standard zip implementation for size and speed.

## internals

- huffman: uses canonical huffman to minimize tree overhead in the archive.
- lz77: sliding window implementation with a custom bit-stream format.
- metadata: uses a custom binary header with `crc32` checksums for every file.
- bit-io: custom bit-level reader and writer for precise stream control.

## testing
`cargo test`
