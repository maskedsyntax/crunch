use std::io::{self, Read, Write};

/// Writes individual bits to an underlying writer.
pub struct BitWriter<W: Write> {
    inner: W,
    buffer: u8,
    bits_count: u8,
}

impl<W: Write> BitWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            buffer: 0,
            bits_count: 0,
        }
    }

    /// Writes a single bit (0 or 1).
    pub fn write_bit(&mut self, bit: bool) -> io::Result<()> {
        if bit {
            self.buffer |= 1 << (7 - self.bits_count);
        }
        self.bits_count += 1;

        if self.bits_count == 8 {
            self.inner.write_all(&[self.buffer])?;
            self.buffer = 0;
            self.bits_count = 0;
        }
        Ok(())
    }

    /// Writes multiple bits from a u64.
    pub fn write_bits(&mut self, value: u64, count: u8) -> io::Result<()> {
        for i in (0..count).rev() {
            let bit = (value >> i) & 1 == 1;
            self.write_bit(bit)?;
        }
        Ok(())
    }

    /// Flushes any remaining bits in the buffer, padding with zeros.
    pub fn flush_bits(&mut self) -> io::Result<()> {
        if self.bits_count > 0 {
            self.inner.write_all(&[self.buffer])?;
            self.buffer = 0;
            self.bits_count = 0;
        }
        self.inner.flush()
    }
}

/// Reads individual bits from an underlying reader.
pub struct BitReader<R: Read> {
    inner: R,
    buffer: u8,
    bits_count: u8,
}

impl<R: Read> BitReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            buffer: 0,
            bits_count: 0,
        }
    }

    /// Reads a single bit. Returns None if EOF is reached.
    pub fn read_bit(&mut self) -> io::Result<Option<bool>> {
        if self.bits_count == 0 {
            let mut buf = [0u8; 1];
            match self.inner.read_exact(&mut buf) {
                Ok(_) => {
                    self.buffer = buf[0];
                    self.bits_count = 8;
                }
                Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => return Ok(None),
                Err(e) => return Err(e),
            }
        }

        let bit = (self.buffer >> (self.bits_count - 1)) & 1 == 1;
        self.bits_count -= 1;
        Ok(Some(bit))
    }

    /// Reads multiple bits and returns them as a u64.
    pub fn read_bits(&mut self, count: u8) -> io::Result<Option<u64>> {
        let mut result = 0u64;
        for _ in 0..count {
            match self.read_bit()? {
                Some(bit) => {
                    result <<= 1;
                    if bit {
                        result |= 1;
                    }
                }
                None => return Ok(None),
            }
        }
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_bit_writer_reader() -> io::Result<()> {
        let mut buffer = Vec::new();
        {
            let mut writer = BitWriter::new(&mut buffer);
            writer.write_bit(true)?;
            writer.write_bit(false)?;
            writer.write_bit(true)?;
            writer.write_bit(true)?;
            writer.write_bits(0b101, 3)?; // 1, 0, 1
            writer.flush_bits()?;
        }

        let mut reader = BitReader::new(Cursor::new(buffer));
        assert_eq!(reader.read_bit()?, Some(true));
        assert_eq!(reader.read_bit()?, Some(false));
        assert_eq!(reader.read_bit()?, Some(true));
        assert_eq!(reader.read_bit()?, Some(true));
        assert_eq!(reader.read_bit()?, Some(true));
        assert_eq!(reader.read_bit()?, Some(false));
        assert_eq!(reader.read_bit()?, Some(true));
        // The last bit should be 0 due to padding if we read it
        assert_eq!(reader.read_bit()?, Some(false));
        assert_eq!(reader.read_bit()?, None);
        Ok(())
    }
}
