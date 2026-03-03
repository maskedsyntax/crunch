use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use std::fs::File;
use std::io::Read;

pub fn print_frequency_histogram<P: AsRef<Path>>(input: P) -> Result<()> {
    let mut file = File::open(input)?;
    let mut content = Vec::new();
    file.read_to_end(&mut content)?;

    let mut frequencies = HashMap::new();
    for &byte in &content {
        *frequencies.entry(byte).or_insert(0) += 1;
    }

    let mut sorted_freqs: Vec<(u8, u64)> = frequencies.into_iter().collect();
    sorted_freqs.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{:<10} {:<10} {:<10}", "Byte", "Char", "Frequency");
    println!("{:-<30}", "");

    let max_freq = sorted_freqs[0].1;
    let bar_width = 40;

    for (byte, freq) in sorted_freqs.iter().take(20) {
        let char_repr = if *byte >= 32 && *byte <= 126 {
            (*byte as char).to_string()
        } else {
            ".".to_string()
        };

        let width = (freq * bar_width / max_freq) as usize;
        let bar = "#".repeat(width);
        println!("{:<10} {:<10} {:<10} {}", byte, char_repr, freq, bar);
    }

    if sorted_freqs.len() > 20 {
        println!("... and {} more symbols", sorted_freqs.len() - 20);
    }

    Ok(())
}
