//! The central directory of a zip (JAR) file: entry names with their CRC-32 and size, read without
//! inflating anything. Enough to recognize a known JAR by its contents.

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ZipEntry {
    pub name: String,
    pub crc32: u32,
    pub size: u64,
}

const EOCD_SIGNATURE: u32 = 0x0605_4b50;
const ZIP64_LOCATOR_SIGNATURE: u32 = 0x0706_4b50;
const ZIP64_EOCD_SIGNATURE: u32 = 0x0606_4b50;
const CENTRAL_HEADER_SIGNATURE: u32 = 0x0201_4b50;
/// The end record (22 bytes) plus the longest comment.
const MAX_EOCD_SEARCH: u64 = 22 + 0xFFFF;

pub fn read_zip_directory(path: &Path) -> io::Result<Vec<ZipEntry>> {
    let mut file = File::open(path)?;
    let len = file.metadata()?.len();
    let tail_len = len.min(MAX_EOCD_SEARCH);
    let tail = read_at(&mut file, len - tail_len, tail_len as usize)?;
    let eocd = (0..tail.len().saturating_sub(21)).rev().find(|&i| u32_at(&tail, i) == EOCD_SIGNATURE).ok_or_else(|| invalid("no end of central directory"))?;
    let mut entries = u64::from(u16_at(&tail, eocd + 10));
    let mut size = u64::from(u32_at(&tail, eocd + 12));
    let mut offset = u64::from(u32_at(&tail, eocd + 16));
    if eocd >= 20 && u32_at(&tail, eocd - 20) == ZIP64_LOCATOR_SIGNATURE {
        let record = read_at(&mut file, u64_at(&tail, eocd - 12), 56)?;
        if u32_at(&record, 0) != ZIP64_EOCD_SIGNATURE {
            return Err(invalid("bad zip64 end of central directory"));
        }
        (entries, size, offset) = (u64_at(&record, 32), u64_at(&record, 40), u64_at(&record, 48));
    }
    let directory = read_at(&mut file, offset, usize::try_from(size).map_err(|_| invalid("directory too large"))?)?;
    parse_entries(&directory, entries)
}

fn parse_entries(directory: &[u8], count: u64) -> io::Result<Vec<ZipEntry>> {
    let mut entries = Vec::new();
    let mut at = 0;
    for _ in 0..count {
        if at + 46 > directory.len() || u32_at(directory, at) != CENTRAL_HEADER_SIGNATURE {
            return Err(invalid("bad central directory header"));
        }
        let name_len = usize::from(u16_at(directory, at + 28));
        let extra_len = usize::from(u16_at(directory, at + 30));
        let comment_len = usize::from(u16_at(directory, at + 32));
        let name = directory.get(at + 46..at + 46 + name_len).ok_or_else(|| invalid("truncated entry name"))?;
        entries.push(ZipEntry {
            name: String::from_utf8_lossy(name).into_owned(),
            crc32: u32_at(directory, at + 16),
            size: u64::from(u32_at(directory, at + 24)),
        });
        at += 46 + name_len + extra_len + comment_len;
    }
    Ok(entries)
}

fn read_at(file: &mut File, offset: u64, len: usize) -> io::Result<Vec<u8>> {
    let mut bytes = vec![0; len];
    file.seek(SeekFrom::Start(offset))?;
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.to_owned())
}

fn u16_at(b: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([b[i], b[i + 1]])
}

fn u32_at(b: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]])
}

fn u64_at(b: &[u8], i: usize) -> u64 {
    u64::from_le_bytes(b[i..i + 8].try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stored, empty-content zip with the given entry names (only what the reader looks at).
    fn zip(names: &[&str]) -> Vec<u8> {
        let mut directory = Vec::new();
        for (i, name) in names.iter().enumerate() {
            directory.extend_from_slice(&CENTRAL_HEADER_SIGNATURE.to_le_bytes());
            directory.extend_from_slice(&[0; 12]);
            directory.extend_from_slice(&(i as u32 + 7).to_le_bytes());
            directory.extend_from_slice(&[0; 4]);
            directory.extend_from_slice(&(i as u32 * 10).to_le_bytes());
            directory.extend_from_slice(&(name.len() as u16).to_le_bytes());
            directory.extend_from_slice(&[0; 16]);
            directory.extend_from_slice(name.as_bytes());
        }
        let mut bytes = b"junk before the directory".to_vec();
        let offset = bytes.len() as u32;
        bytes.extend_from_slice(&directory);
        bytes.extend_from_slice(&EOCD_SIGNATURE.to_le_bytes());
        bytes.extend_from_slice(&[0; 6]);
        bytes.extend_from_slice(&(names.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&(directory.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&offset.to_le_bytes());
        bytes.extend_from_slice(&[0; 2]);
        bytes
    }

    #[test]
    fn reads_names_crcs_and_sizes() {
        let dir = std::env::temp_dir().join(format!("ktrs-zip-directory-{}", std::process::id()));
        std::fs::write(&dir, zip(&["META-INF/MANIFEST.MF", "a/B.class"])).unwrap();
        let entries = read_zip_directory(&dir).unwrap();
        let _ = std::fs::remove_file(&dir);
        assert_eq!(
            entries,
            [
                ZipEntry { name: "META-INF/MANIFEST.MF".into(), crc32: 7, size: 0 },
                ZipEntry { name: "a/B.class".into(), crc32: 8, size: 10 },
            ]
        );
    }
}
