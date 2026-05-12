use std::{
    fmt, fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Context, Result};
use flate2::read::ZlibDecoder;

const XP3_MAGIC: &[u8] = b"XP3\r\n \n\x1a\x8bg\x01";
#[cfg(test)]
const XP3_HEADER_LEN: usize = 0x13;
const CHUNK_HEADER_LEN: usize = 12;
const INDEX_KIND_RAW: u8 = 0;
const INDEX_KIND_ZLIB: u8 = 1;

#[derive(Debug, Clone)]
pub struct Xp3Archive {
    path: PathBuf,
    entries: Vec<Xp3Entry>,
    options: Xp3Options,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Xp3Entry {
    pub name: String,
    pub encrypted: bool,
    pub original_size: u64,
    pub packed_size: u64,
    pub checksum: Option<u32>,
    pub segments: Vec<Xp3Segment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Xp3Segment {
    pub compressed: bool,
    pub offset: u64,
    pub original_size: u64,
    pub packed_size: u64,
}

pub trait Xp3CryptScheme: fmt::Debug + Send + Sync {
    fn decrypt_name_bytes(&self, _bytes: &mut [u8]) {}

    fn decrypt_segment_bytes(&self, _bytes: &mut [u8], _entry: &Xp3Entry, _segment: &Xp3Segment) {}
}

#[derive(Debug, Clone, Default)]
pub struct Xp3Options {
    pub decryptor: Xp3Decryptor,
}

#[derive(Debug, Clone, Default)]
pub enum Xp3Decryptor {
    #[default]
    None,
    Xor {
        key: u8,
    },
    NameXor {
        key: u8,
    },
    Custom {
        scheme: Arc<dyn Xp3CryptScheme>,
    },
}

impl Xp3Decryptor {
    fn decrypt_name_bytes(&self, bytes: &mut [u8]) {
        match self {
            Self::NameXor { key } => xor_bytes(bytes, *key),
            Self::Custom { scheme } => scheme.decrypt_name_bytes(bytes),
            Self::None | Self::Xor { .. } => {}
        }
    }

    fn decrypt_segment_bytes(&self, bytes: &mut [u8], entry: &Xp3Entry, segment: &Xp3Segment) {
        match self {
            Self::Xor { key } => xor_bytes(bytes, *key),
            Self::Custom { scheme } => scheme.decrypt_segment_bytes(bytes, entry, segment),
            Self::None | Self::NameXor { .. } => {}
        }
    }
}

impl Xp3Archive {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_file_with_options(path, Xp3Options::default())
    }

    pub fn from_file_with_options(path: impl AsRef<Path>, options: Xp3Options) -> Result<Self> {
        let path = path.as_ref();
        let entries = parse_xp3_entries_from_file(path, &options)
            .with_context(|| format!("failed to parse XP3 {}", path.display()))?;

        Ok(Self {
            path: path.to_owned(),
            entries,
            options,
        })
    }

    pub fn entries(&self) -> &[Xp3Entry] {
        &self.entries
    }

    pub fn find(&self, name: &str) -> Option<&Xp3Entry> {
        let normalized = normalize_path(name);
        self.entries
            .iter()
            .find(|entry| normalize_path(&entry.name) == normalized)
    }

    pub fn read_file(&self, name: &str) -> Result<Vec<u8>> {
        let entry = self
            .find(name)
            .with_context(|| format!("XP3 entry `{name}` not found"))?;
        self.read_entry(entry)
    }

    pub fn read_entry(&self, entry: &Xp3Entry) -> Result<Vec<u8>> {
        let archive = fs::read(&self.path)
            .with_context(|| format!("failed to read {}", self.path.display()))?;
        read_entry_from_bytes(&archive, entry, &self.options)
    }
}

fn parse_xp3_entries_from_file(path: &Path, options: &Xp3Options) -> Result<Vec<Xp3Entry>> {
    let mut file =
        fs::File::open(path).with_context(|| format!("failed to open XP3 {}", path.display()))?;
    let base_offset = find_xp3_header_in_file(&mut file)?.context("XP3 header not found")?;
    let index_offset = resolve_index_offset_from_file(&mut file, base_offset)?;
    let index = read_index_chain_from_file(&mut file, base_offset, index_offset)?;
    parse_index(&index, base_offset as u64, options)
}

fn find_xp3_header(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(XP3_MAGIC.len())
        .position(|window| window == XP3_MAGIC)
}

fn find_xp3_header_in_file(file: &mut fs::File) -> Result<Option<usize>> {
    file.seek(SeekFrom::Start(0))?;
    let mut head = vec![0; 1024 * 1024];
    let len = file.read(&mut head)?;
    head.truncate(len);
    Ok(find_xp3_header(&head))
}

fn resolve_index_offset_from_file(file: &mut fs::File, base_offset: usize) -> Result<u64> {
    let index_pointer = base_offset + XP3_MAGIC.len();
    let index_offset = base_offset as u64 + read_u64_at_file(file, index_pointer as u64)?;
    if index_offset >= file.metadata()?.len() {
        bail!("XP3 index offset exceeds archive size");
    }
    Ok(index_offset)
}

fn read_index_chain_from_file(
    file: &mut fs::File,
    base_offset: usize,
    mut offset: u64,
) -> Result<Vec<u8>> {
    let mut index = Vec::new();
    let mut seen_offsets = Vec::new();
    let file_size = file.metadata()?.len();

    loop {
        if seen_offsets.contains(&offset) {
            bail!("XP3 index chain contains a loop");
        }
        seen_offsets.push(offset);

        let (mut chunk, next_offset) = read_index_chunk_from_file(file, offset)?;
        index.append(&mut chunk);

        let Some(next_offset) = next_offset else {
            break;
        };
        offset = base_offset as u64 + next_offset;
        if offset >= file_size {
            bail!("XP3 chained index offset exceeds archive size");
        }
    }

    Ok(index)
}

fn read_index_chunk_from_file(file: &mut fs::File, offset: u64) -> Result<(Vec<u8>, Option<u64>)> {
    file.seek(SeekFrom::Start(offset))?;
    let mut kind = [0_u8; 1];
    file.read_exact(&mut kind)
        .context("XP3 index kind is missing")?;
    let has_next = kind[0] & 0x80 != 0;
    let kind = kind[0] & 0x7f;
    let (index, next_pointer_offset) = match kind {
        INDEX_KIND_RAW => {
            let size = read_u64_from_reader(file)? as usize;
            let mut index = vec![0; size];
            file.read_exact(&mut index)
                .context("failed to read XP3 raw index")?;
            (index, offset + 9 + size as u64)
        }
        INDEX_KIND_ZLIB => {
            let packed_size = read_u64_from_reader(file)? as usize;
            let unpacked_size = read_u64_from_reader(file)? as usize;
            let mut packed = vec![0; packed_size];
            file.read_exact(&mut packed)
                .context("failed to read XP3 packed index")?;
            let mut decoder = ZlibDecoder::new(packed.as_slice());
            let mut decoded = Vec::with_capacity(unpacked_size);
            decoder
                .read_to_end(&mut decoded)
                .context("failed to inflate XP3 index")?;
            if decoded.len() != unpacked_size {
                bail!("XP3 index unpacked size mismatch");
            }
            (decoded, offset + 17 + packed_size as u64)
        }
        _ => bail!("unsupported XP3 index kind {kind}"),
    };

    let next_offset = if has_next {
        Some(read_u64_at_file(file, next_pointer_offset)?)
    } else {
        None
    };
    Ok((index, next_offset))
}

fn parse_index(index: &[u8], base_offset: u64, options: &Xp3Options) -> Result<Vec<Xp3Entry>> {
    let mut reader = ByteReader::new(index);
    let mut entries = Vec::new();

    while reader.remaining() >= CHUNK_HEADER_LEN {
        let chunk = reader.read_tag()?;
        let chunk_size = reader.read_u64()? as usize;
        let chunk_bytes = reader.read_bytes(chunk_size)?;
        if chunk == *b"File" {
            if let Some(entry) = parse_file_chunk(chunk_bytes, base_offset, options)? {
                entries.push(entry);
            }
        }
    }

    Ok(entries)
}

fn parse_file_chunk(
    chunk: &[u8],
    base_offset: u64,
    options: &Xp3Options,
) -> Result<Option<Xp3Entry>> {
    let mut reader = ByteReader::new(chunk);
    let mut name = None;
    let mut original_size = 0;
    let mut packed_size = 0;
    let mut checksum = None;
    let mut segments = Vec::new();
    let mut encrypted = false;

    while reader.remaining() >= CHUNK_HEADER_LEN {
        let section = reader.read_tag()?;
        let section_size = reader.read_u64()? as usize;
        let section_bytes = reader.read_bytes(section_size)?;
        match &section {
            b"info" => {
                let mut info = ByteReader::new(section_bytes);
                encrypted = info.read_u32()? != 0;
                original_size = info.read_u64()?;
                packed_size = info.read_u64()?;
                name = Some(info.read_utf16_string(&options.decryptor)?);
            }
            b"segm" => {
                let mut segm = ByteReader::new(section_bytes);
                while segm.remaining() >= 28 {
                    let compressed = segm.read_u32()? != 0;
                    let offset = base_offset + segm.read_u64()?;
                    let original_size = segm.read_u64()?;
                    let packed_size = segm.read_u64()?;
                    segments.push(Xp3Segment {
                        compressed,
                        offset,
                        original_size,
                        packed_size,
                    });
                }
            }
            b"adlr" if section_bytes.len() >= 4 => {
                checksum = Some(u32::from_le_bytes(
                    section_bytes[..4]
                        .try_into()
                        .expect("slice length is fixed"),
                ));
            }
            _ => {}
        }
    }

    let Some(name) = name.filter(|name| !name.is_empty()) else {
        return Ok(None);
    };
    if segments.is_empty() {
        return Ok(None);
    }

    Ok(Some(Xp3Entry {
        name,
        encrypted,
        original_size,
        packed_size,
        checksum,
        segments,
    }))
}

fn read_entry_from_bytes(
    archive: &[u8],
    entry: &Xp3Entry,
    options: &Xp3Options,
) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(entry.original_size as usize);
    for segment in &entry.segments {
        let start = segment.offset as usize;
        let end = start
            .checked_add(segment.packed_size as usize)
            .context("XP3 segment size overflow")?;
        if end > archive.len() {
            bail!("XP3 segment for `{}` exceeds archive size", entry.name);
        }

        let mut segment_bytes = archive[start..end].to_vec();
        if entry.encrypted {
            options
                .decryptor
                .decrypt_segment_bytes(&mut segment_bytes, entry, segment);
        }

        if segment.compressed {
            let mut decoder = ZlibDecoder::new(segment_bytes.as_slice());
            let before = output.len();
            decoder
                .read_to_end(&mut output)
                .with_context(|| format!("failed to inflate XP3 segment `{}`", entry.name))?;
            if output.len() - before != segment.original_size as usize {
                bail!("XP3 segment unpacked size mismatch for `{}`", entry.name);
            }
        } else {
            if segment.original_size != segment.packed_size {
                bail!("stored XP3 segment size mismatch for `{}`", entry.name);
            }
            output.extend_from_slice(&segment_bytes);
        }
    }

    if output.len() as u64 != entry.original_size {
        bail!("XP3 entry size mismatch for `{}`", entry.name);
    }
    Ok(output)
}

fn xor_bytes(bytes: &mut [u8], key: u8) {
    for byte in bytes {
        *byte ^= key;
    }
}

fn read_u64_at_file(file: &mut fs::File, offset: u64) -> Result<u64> {
    file.seek(SeekFrom::Start(offset))?;
    read_u64_from_reader(file)
}

fn read_u64_from_reader(reader: &mut impl Read) -> Result<u64> {
    let mut bytes = [0_u8; 8];
    reader
        .read_exact(&mut bytes)
        .context("XP3 u64 is out of bounds")?;
    Ok(u64::from_le_bytes(bytes))
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/").to_ascii_lowercase()
}

pub fn xp3_magic() -> &'static [u8] {
    XP3_MAGIC
}

struct ByteReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ByteReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn read_tag(&mut self) -> Result<[u8; 4]> {
        Ok(self
            .read_bytes(4)?
            .try_into()
            .expect("slice length is fixed"))
    }

    fn read_u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.read_bytes(2)?
                .try_into()
                .expect("slice length is fixed"),
        ))
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.read_bytes(4)?
                .try_into()
                .expect("slice length is fixed"),
        ))
    }

    fn read_u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(
            self.read_bytes(8)?
                .try_into()
                .expect("slice length is fixed"),
        ))
    }

    fn read_utf16_string(&mut self, decryptor: &Xp3Decryptor) -> Result<String> {
        let char_count = self.read_u16()? as usize;
        let raw_len = char_count
            .checked_mul(2)
            .context("XP3 UTF-16 name length overflow")?;
        let mut raw = self.read_bytes(raw_len)?.to_vec();
        decryptor.decrypt_name_bytes(&mut raw);
        let mut chars = Vec::with_capacity(char_count);
        for chunk in raw.chunks_exact(2) {
            chars.push(u16::from_le_bytes(
                chunk.try_into().expect("slice length is fixed"),
            ));
        }
        String::from_utf16(&chars).context("invalid XP3 UTF-16 file name")
    }

    fn read_bytes(&mut self, len: usize) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(len)
            .context("XP3 reader offset overflow")?;
        let slice = self
            .bytes
            .get(self.offset..end)
            .context("unexpected end of XP3 data")?;
        self.offset = end;
        Ok(slice)
    }
}

#[cfg(test)]
mod tests {
    use flate2::{write::ZlibEncoder, Compression};
    use std::io::Write;

    use super::*;

    #[test]
    fn xp3_reads_stored_entry() {
        let root = test_dir("suzu-xp3-stored");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("stored.xp3");
        write_test_xp3(&path, "scenario/start.ks", b"hello xp3", false, false);

        let archive = Xp3Archive::from_file(&path).unwrap();

        assert_eq!(archive.entries()[0].name, "scenario/start.ks");
        assert_eq!(
            archive.read_file("scenario/start.ks").unwrap(),
            b"hello xp3"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn xp3_reads_compressed_index_and_segment() {
        let root = test_dir("suzu-xp3-compressed");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("compressed.xp3");
        write_test_xp3(&path, "image/bg.png", b"fake png payload", true, true);

        let archive = Xp3Archive::from_file(&path).unwrap();

        assert_eq!(
            archive.read_file("IMAGE\\BG.PNG").unwrap(),
            b"fake png payload"
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn xp3_reads_xor_encrypted_segment() {
        let root = test_dir("suzu-xp3-xor");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("encrypted.xp3");
        write_test_xp3_with_encryption(&path, "voice/line.ogg", b"voice bytes", 0x5a);

        let archive = Xp3Archive::from_file_with_options(
            &path,
            Xp3Options {
                decryptor: Xp3Decryptor::Xor { key: 0x5a },
            },
        )
        .unwrap();

        assert!(archive.entries()[0].encrypted);
        assert_eq!(archive.read_file("voice/line.ogg").unwrap(), b"voice bytes");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn xp3_reads_chained_index() {
        let root = test_dir("suzu-xp3-chained");
        fs::create_dir_all(&root).unwrap();
        let path = root.join("chained.xp3");
        write_test_xp3_with_chained_index(&path, "main/start.ks", b"chain payload");

        let archive = Xp3Archive::from_file(&path).unwrap();

        assert_eq!(archive.entries().len(), 2);
        assert_eq!(
            archive.read_file("main/start.ks").unwrap(),
            b"chain payload"
        );

        let _ = fs::remove_dir_all(root);
    }

    fn write_test_xp3(
        path: &Path,
        name: &str,
        data: &[u8],
        compress_index: bool,
        compress_segment: bool,
    ) {
        let segment = if compress_segment {
            zlib(data)
        } else {
            data.to_vec()
        };
        let segment_offset = XP3_HEADER_LEN as u64;
        let index_offset = segment_offset + segment.len() as u64;
        let index = build_index(
            name,
            data.len() as u64,
            &segment,
            segment_offset,
            compress_segment,
        );
        let packed_index = if compress_index {
            zlib(&index)
        } else {
            index.clone()
        };

        let mut bytes = XP3_MAGIC.to_vec();
        bytes.extend_from_slice(&index_offset.to_le_bytes());
        bytes.extend_from_slice(&segment);
        bytes.push(if compress_index {
            INDEX_KIND_ZLIB
        } else {
            INDEX_KIND_RAW
        });
        bytes.extend_from_slice(&(packed_index.len() as u64).to_le_bytes());
        if compress_index {
            bytes.extend_from_slice(&(index.len() as u64).to_le_bytes());
        }
        bytes.extend_from_slice(&packed_index);
        fs::write(path, bytes).unwrap();
    }

    fn write_test_xp3_with_chained_index(path: &Path, name: &str, data: &[u8]) {
        let segment_offset = XP3_HEADER_LEN as u64;
        let index_offset = segment_offset + data.len() as u64;
        let warning_index = build_index("warning.txt", 7, b"warning", segment_offset, false);
        let main_index_offset = index_offset + 9 + warning_index.len() as u64 + 8;
        let main_index = build_index(name, data.len() as u64, data, segment_offset, false);
        let packed_main_index = zlib(&main_index);

        let mut bytes = XP3_MAGIC.to_vec();
        bytes.extend_from_slice(&index_offset.to_le_bytes());
        bytes.extend_from_slice(data);
        bytes.push(0x80 | INDEX_KIND_RAW);
        bytes.extend_from_slice(&(warning_index.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&warning_index);
        bytes.extend_from_slice(&main_index_offset.to_le_bytes());
        bytes.push(INDEX_KIND_ZLIB);
        bytes.extend_from_slice(&(packed_main_index.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&(main_index.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&packed_main_index);
        fs::write(path, bytes).unwrap();
    }

    fn write_test_xp3_with_encryption(path: &Path, name: &str, data: &[u8], key: u8) {
        let mut segment = data.to_vec();
        xor_bytes(&mut segment, key);
        let segment_offset = XP3_HEADER_LEN as u64;
        let index_offset = segment_offset + segment.len() as u64;
        let index = build_index_with_encryption(name, data.len() as u64, &segment, segment_offset);

        let mut bytes = XP3_MAGIC.to_vec();
        bytes.extend_from_slice(&index_offset.to_le_bytes());
        bytes.extend_from_slice(&segment);
        bytes.push(INDEX_KIND_RAW);
        bytes.extend_from_slice(&(index.len() as u64).to_le_bytes());
        bytes.extend_from_slice(&index);
        fs::write(path, bytes).unwrap();
    }

    fn build_index(
        name: &str,
        original_size: u64,
        packed_segment: &[u8],
        segment_offset: u64,
        compressed_segment: bool,
    ) -> Vec<u8> {
        let mut info = Vec::new();
        info.extend_from_slice(&0_u32.to_le_bytes());
        info.extend_from_slice(&original_size.to_le_bytes());
        info.extend_from_slice(&(packed_segment.len() as u64).to_le_bytes());
        let name_utf16 = name.encode_utf16().collect::<Vec<_>>();
        info.extend_from_slice(&(name_utf16.len() as u16).to_le_bytes());
        for ch in name_utf16 {
            info.extend_from_slice(&ch.to_le_bytes());
        }

        let mut segm = Vec::new();
        segm.extend_from_slice(&(u32::from(compressed_segment)).to_le_bytes());
        segm.extend_from_slice(&segment_offset.to_le_bytes());
        segm.extend_from_slice(&original_size.to_le_bytes());
        segm.extend_from_slice(&(packed_segment.len() as u64).to_le_bytes());

        let mut file = Vec::new();
        push_chunk(&mut file, b"info", &info);
        push_chunk(&mut file, b"segm", &segm);

        let mut index = Vec::new();
        push_chunk(&mut index, b"File", &file);
        index
    }

    fn build_index_with_encryption(
        name: &str,
        original_size: u64,
        packed_segment: &[u8],
        segment_offset: u64,
    ) -> Vec<u8> {
        let mut info = Vec::new();
        info.extend_from_slice(&1_u32.to_le_bytes());
        info.extend_from_slice(&original_size.to_le_bytes());
        info.extend_from_slice(&(packed_segment.len() as u64).to_le_bytes());
        let name_utf16 = name.encode_utf16().collect::<Vec<_>>();
        info.extend_from_slice(&(name_utf16.len() as u16).to_le_bytes());
        for ch in name_utf16 {
            info.extend_from_slice(&ch.to_le_bytes());
        }

        let mut segm = Vec::new();
        segm.extend_from_slice(&0_u32.to_le_bytes());
        segm.extend_from_slice(&segment_offset.to_le_bytes());
        segm.extend_from_slice(&original_size.to_le_bytes());
        segm.extend_from_slice(&(packed_segment.len() as u64).to_le_bytes());

        let mut file = Vec::new();
        push_chunk(&mut file, b"info", &info);
        push_chunk(&mut file, b"segm", &segm);

        let mut index = Vec::new();
        push_chunk(&mut index, b"File", &file);
        index
    }

    fn push_chunk(output: &mut Vec<u8>, tag: &[u8; 4], body: &[u8]) {
        output.extend_from_slice(tag);
        output.extend_from_slice(&(body.len() as u64).to_le_bytes());
        output.extend_from_slice(body);
    }

    fn zlib(data: &[u8]) -> Vec<u8> {
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(data).unwrap();
        encoder.finish().unwrap()
    }

    fn test_dir(name: &str) -> PathBuf {
        let mut root = std::env::temp_dir();
        root.push(format!(
            "{name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&root);
        root
    }
}
