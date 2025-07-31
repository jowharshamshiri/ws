use anyhow::{Context, Result};
use content_inspector::{ContentType, inspect};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

/// Detects if a file is binary or text
pub struct BinaryDetector {
    /// Maximum number of bytes to read for detection
    max_sample_size: usize,
    /// Threshold for binary detection (percentage of non-printable characters)
    binary_threshold: f64,
}

impl Default for BinaryDetector {
    fn default() -> Self {
        Self {
            max_sample_size: 8192, // 8KB sample
            binary_threshold: 0.3,  // 30% non-printable = binary
        }
    }
}

impl BinaryDetector {
    pub fn new(max_sample_size: usize, binary_threshold: f64) -> Self {
        Self {
            max_sample_size,
            binary_threshold,
        }
    }

    /// Check if a file is binary using multiple detection methods with extension fail-safe
    pub fn is_binary<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        let path = path.as_ref();
        
        // First, check file extension for known binary types (fail-safe for important files)
        if self.is_binary_by_extension(path) {
            return Ok(true);
        }

        // Check for binary signatures early (before content_inspector which can be fooled by compressed data)
        if let Ok(is_binary_by_signature) = self.check_binary_signature_only(path) {
            if is_binary_by_signature {
                return Ok(true);
            }
        }

        // Check using content_inspector crate (fast method)
        if let Ok(content_type) = self.detect_by_content_inspector(path) {
            match content_type {
                ContentType::BINARY => return Ok(true),
                ContentType::UTF_8 | ContentType::UTF_8_BOM | 
                ContentType::UTF_16LE | ContentType::UTF_16BE |
                ContentType::UTF_32LE | ContentType::UTF_32BE => return Ok(false),
            }
        }

        // Fallback to manual analysis with enhanced binary signatures
        self.is_binary_by_content_analysis(path)
    }


    /// Check if file is likely binary based on file extension (fail-safe)
    fn is_binary_by_extension(&self, path: &Path) -> bool {
        if let Some(extension) = path.extension() {
            if let Some(ext_str) = extension.to_str() {
                let ext_lower = ext_str.to_lowercase();
                return BINARY_EXTENSIONS.contains(&ext_lower.as_str());
            }
        }
        false
    }

    /// Check only for binary signatures (used before content_inspector to catch compressed data)
    fn check_binary_signature_only(&self, path: &Path) -> Result<bool> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open file for signature check: {}", path.display()))?;
        
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; 32]; // Only need first 32 bytes for signatures
        
        let bytes_read = reader.read(&mut buffer)
            .with_context(|| format!("Failed to read file for signature check: {}", path.display()))?;
        
        if bytes_read == 0 {
            return Ok(false); // Empty files are treated as text
        }

        buffer.truncate(bytes_read);
        Ok(self.has_binary_signature(&buffer))
    }

    /// Use content_inspector crate for fast detection
    fn detect_by_content_inspector(&self, path: &Path) -> Result<ContentType> {
        let mut file = File::open(path)
            .with_context(|| format!("Failed to open file for binary detection: {}", path.display()))?;
        
        let mut buffer = vec![0; self.max_sample_size];
        let bytes_read = file.read(&mut buffer)
            .with_context(|| format!("Failed to read file for binary detection: {}", path.display()))?;
        
        buffer.truncate(bytes_read);
        Ok(inspect(&buffer))
    }

    /// Manual content analysis for edge cases
    fn is_binary_by_content_analysis(&self, path: &Path) -> Result<bool> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open file for content analysis: {}", path.display()))?;
        
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; self.max_sample_size];
        
        let bytes_read = reader.read(&mut buffer)
            .with_context(|| format!("Failed to read file for content analysis: {}", path.display()))?;
        
        if bytes_read == 0 {
            return Ok(false); // Empty files are treated as text
        }

        buffer.truncate(bytes_read);
        
        // Check for common binary file signatures
        if self.has_binary_signature(&buffer) {
            return Ok(true);
        }
        
        // Check for null bytes (strong indicator of binary)
        if buffer.contains(&0) {
            return Ok(true);
        }

        // Try to validate as UTF-8 first
        if let Ok(text) = std::str::from_utf8(&buffer) {
            // If it's valid UTF-8, count actual non-printable characters (excluding valid Unicode)
            let non_printable_count = text.chars()
                .filter(|&c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
                .count();
            let non_printable_ratio = non_printable_count as f64 / text.chars().count() as f64;
            return Ok(non_printable_ratio > self.binary_threshold);
        }

        // If not valid UTF-8, count non-printable bytes (old method)
        let non_printable_count = buffer.iter()
            .filter(|&&byte| !is_printable_ascii(byte) && !is_valid_utf8_start(byte))
            .count();

        let non_printable_ratio = non_printable_count as f64 / bytes_read as f64;
        Ok(non_printable_ratio > self.binary_threshold)
    }

    /// Check for common binary file signatures at the beginning of the file
    fn has_binary_signature(&self, buffer: &[u8]) -> bool {
        if buffer.len() < 2 {
            return false;
        }
        
        // Check for Protocol Buffer patterns first (they can be tricky)
        if self.is_protocol_buffer(buffer) {
            return true;
        }
        
        // Check for common binary signatures
        match &buffer[0..2] {
            // zlib compression (used by Git objects)
            [0x78, 0x01] | [0x78, 0x9c] | [0x78, 0xda] => true,
            // PNG
            [0x89, 0x50] if buffer.len() >= 8 && &buffer[0..8] == b"\x89PNG\r\n\x1a\n" => true,
            // JPEG
            [0xff, 0xd8] => true,
            // PDF
            [0x25, 0x50] if buffer.len() >= 4 && &buffer[0..4] == b"%PDF" => true,
            // ZIP/JAR/etc
            [0x50, 0x4b] => true,
            // ELF executable
            [0x7f, 0x45] if buffer.len() >= 4 && &buffer[0..4] == b"\x7fELF" => true,
            // Windows PE/COFF executable
            [0x4d, 0x5a] => true,
            // Mach-O executable (macOS)
            [0xfe, 0xed] | [0xfe, 0xec] | [0xce, 0xfa] | [0xcf, 0xfa] => true,
            // TAR
            _ if buffer.len() >= 262 && &buffer[257..262] == b"ustar" => true,
            // GZIP
            [0x1f, 0x8b] => true,
            // BZIP2
            [0x42, 0x5a] if buffer.len() >= 3 && buffer[2] == 0x68 => true,
            // XZ
            [0xfd, 0x37] if buffer.len() >= 6 && &buffer[0..6] == b"\xfd7zXZ\x00" => true,
            // 7-Zip
            [0x37, 0x7a] if buffer.len() >= 6 && &buffer[0..6] == b"7z\xbc\xaf\x27\x1c" => true,
            // Class files (Java)
            [0xca, 0xfe] => true,
            // DEX files (Android)
            [0x64, 0x65] if buffer.len() >= 8 && &buffer[0..8] == b"dex\n035\0" => true,
            _ => false,
        }
    }

    /// Detect Protocol Buffer files by their characteristic patterns
    fn is_protocol_buffer(&self, buffer: &[u8]) -> bool {
        if buffer.len() < 8 {
            return false;
        }

        // First, reject if it looks like text (contains mostly printable ASCII or valid UTF-8)
        let ascii_count = buffer.iter().take(64).filter(|&&b| b >= 0x20 && b <= 0x7E).count();
        let ascii_ratio = ascii_count as f64 / buffer.len().min(64) as f64;
        
        // Also check if it's valid UTF-8 (which would indicate text, not binary protobuf)
        let is_valid_utf8 = std::str::from_utf8(buffer).is_ok();
        
        if ascii_ratio > 0.7 || is_valid_utf8 {
            return false; // Too much readable text or valid UTF-8, unlikely to be protobuf
        }

        // Protocol Buffers use varint encoding and have specific wire type patterns
        let mut valid_proto_sequences = 0;
        let mut i = 0;
        
        while i < buffer.len().min(32) { // Check first 32 bytes for protobuf patterns
            let byte = buffer[i];
            
            // Check for valid protobuf wire type patterns
            let wire_type = byte & 0x07;
            let field_number = byte >> 3;
            
            // Valid wire types: 0 (varint), 1 (64-bit), 2 (length-delimited), 5 (32-bit)
            if matches!(wire_type, 0 | 1 | 2 | 5) && field_number > 0 && field_number < 32 {
                match wire_type {
                    0 => {
                        // Varint - check for reasonable varint values
                        if i + 1 < buffer.len() {
                            let next = buffer[i + 1];
                            if next < 0x80 || (next & 0x80 != 0 && next & 0x7F < 0x40) {
                                valid_proto_sequences += 1;
                                i += 2; // Skip the varint value
                                continue;
                            }
                        }
                    },
                    2 => {
                        // Length-delimited - check for reasonable length
                        if i + 1 < buffer.len() {
                            let length = buffer[i + 1];
                            if length > 0 && length < 64 && (i + 2 + length as usize) <= buffer.len() {
                                valid_proto_sequences += 2; // Length-delimited is a strong indicator
                                i += 2 + length as usize; // Skip the entire field
                                continue;
                            }
                        }
                    },
                    1 | 5 => {
                        // Fixed-width fields
                        valid_proto_sequences += 1;
                        i += if wire_type == 1 { 9 } else { 5 }; // Skip field + data
                        continue;
                    },
                    _ => {}
                }
            }
            
            i += 1;
        }
        
        // Require at least 2 valid protobuf sequences and check for non-text characteristics
        let has_null_bytes = buffer[..buffer.len().min(64)].contains(&0);
        let non_ascii_count = buffer.iter().take(64).filter(|&&b| b > 0x7E || (b < 0x20 && !matches!(b, 0x09 | 0x0A | 0x0D))).count();
        let non_ascii_ratio = non_ascii_count as f64 / buffer.len().min(64) as f64;
        
        // Must have valid protobuf patterns AND binary characteristics
        valid_proto_sequences >= 2 && (has_null_bytes || non_ascii_ratio > 0.2)
    }

    /// Check if the file appears to be a text file that we should process
    pub fn is_text_file<P: AsRef<Path>>(&self, path: P) -> Result<bool> {
        Ok(!self.is_binary(path)?)
    }

    /// Get a description of why a file is considered binary
    pub fn get_binary_reason<P: AsRef<Path>>(&self, path: P) -> Result<Option<String>> {
        let path = path.as_ref();
        
        if self.is_binary_by_extension(path) {
            return Ok(Some("Binary file extension".to_string()));
        }

        if let Ok(content_type) = self.detect_by_content_inspector(path) {
            match content_type {
                ContentType::BINARY => return Ok(Some("Content inspection detected binary".to_string())),
                _ => {}
            }
        }

        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buffer = vec![0; self.max_sample_size];
        let bytes_read = reader.read(&mut buffer)?;
        
        if bytes_read == 0 {
            return Ok(None);
        }

        buffer.truncate(bytes_read);
        
        // Check for binary signatures first
        if self.has_binary_signature(&buffer) {
            // Check for Protocol Buffer first since it doesn't have a fixed signature
            if self.is_protocol_buffer(&buffer) {
                return Ok(Some("Detected as Protocol Buffer binary data".to_string()));
            }
            
            let signature_desc = match &buffer[0..2.min(buffer.len())] {
                [0x78, 0x01] | [0x78, 0x9c] | [0x78, 0xda] => "zlib compressed data",
                [0x89, 0x50] => "PNG image",
                [0xff, 0xd8] => "JPEG image", 
                [0x25, 0x50] => "PDF document",
                [0x50, 0x4b] => "ZIP archive",
                [0x7f, 0x45] => "ELF executable",
                [0x4d, 0x5a] => "Windows executable",
                [0xfe, 0xed] | [0xfe, 0xec] | [0xce, 0xfa] | [0xcf, 0xfa] => "Mach-O executable",
                [0x1f, 0x8b] => "GZIP compressed",
                [0x42, 0x5a] => "BZIP2 compressed",
                _ => "binary file signature",
            };
            return Ok(Some(format!("Detected as {}", signature_desc)));
        }
        
        if buffer.contains(&0) {
            return Ok(Some("Contains null bytes".to_string()));
        }

        let non_printable_count = buffer.iter()
            .filter(|&&byte| !is_printable_ascii(byte) && !is_valid_utf8_start(byte))
            .count();

        let non_printable_ratio = non_printable_count as f64 / bytes_read as f64;
        if non_printable_ratio > self.binary_threshold {
            return Ok(Some(format!("High ratio of non-printable characters: {:.1}%", non_printable_ratio * 100.0)));
        }

        Ok(None)
    }
}

/// Check if a byte is printable ASCII
fn is_printable_ascii(byte: u8) -> bool {
    matches!(byte, 0x20..=0x7E | 0x09 | 0x0A | 0x0D) // printable ASCII + tab, newline, carriage return
}

/// Check if a byte could be the start of a valid UTF-8 sequence
fn is_valid_utf8_start(byte: u8) -> bool {
    // UTF-8 start bytes: 0xxxxxxx, 110xxxxx, 1110xxxx, 11110xxx
    byte < 0x80 || (byte >= 0xC0 && byte < 0xF8)
}

/// Comprehensive binary file extensions for diverse workspaces
const BINARY_EXTENSIONS: &[&str] = &[
    // Executables and system files
    "exe", "dll", "so", "dylib", "app", "deb", "rpm", "msi", "dmg", "run", "bin",
    "com", "bat", "cmd", "scr", "sys", "drv", "cpl", "ocx", "axl", "elf",
    
    // Archives and compressed files
    "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "cab", "ace", "arc", "arj",
    "lzh", "lha", "zoo", "cpio", "shar", "lz", "lzma", "tgz", "tbz", "tbz2",
    "txz", "tlz", "Z", "deb", "rpm", "pkg", "apk", "snap", "flatpak", "appimage",
    
    // Images and graphics
    "jpg", "jpeg", "png", "gif", "bmp", "tiff", "tif", "webp", "svg", "ico", "cur",
    "psd", "ai", "eps", "raw", "cr2", "nef", "orf", "sr2", "dng", "exr", "hdr",
    "tga", "pcx", "ppm", "pgm", "pbm", "xbm", "xpm", "jp2", "jpx", "j2k", "heic",
    "heif", "avif", "jxl", "wbmp", "emf", "wmf", "cgm", "pic", "pict",
    
    // Videos and multimedia
    "mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "3gp", "3g2",
    "mpg", "mpeg", "asf", "rm", "rmvb", "vob", "mts", "m2ts", "f4v",
    "ogv", "divx", "xvid", "mxf", "amv", "drc", "gif", "gifv", "mng", "qt",
    "yuv", "roq", "svi", "viv", "vp8", "vp9", "av1",
    
    // Audio files
    "mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "aiff", "au", "ra",
    "opus", "ape", "ac3", "dts", "amr", "awb", "gsm", "m4r", "mpc", "tta",
    "wv", "8svx", "cda", "mid", "midi", "rmi", "kar", "mod", "s3m", "xm",
    "it", "669", "amf", "ams", "dbm", "dmf", "dsm", "far", "mdl", "med",
    "mtm", "okt", "ptm", "stm", "ult", "umx", "mt2", "psm",
    
    // Documents and office files
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "odp",
    "rtf", "pages", "numbers", "key", "pub", "vsd", "vsdx", "mpp", "one",
    "wps", "wpd", "sxw", "sxc", "sxi", "sxd", "stw", "stc", "sti", "std",
    "odf", "otg", "otp", "ots", "ott", "odm", "oth", "odb", "oxt",
    
    // Databases
    "db", "sqlite", "sqlite3", "mdb", "accdb", "dbf", "fdb", "gdb", "nsf",
    "odb", "pdb", "sql", "wdb", "bak", "dmp", "frm", "myd", "myi", "ibd",
    "fpt", "cdx", "idx", "ndx", "mdx", "fox", "prg", "fxp", "dbc", "dct",
    
    // Development and build artifacts
    "o", "obj", "lib", "a", "pdb", "ilk", "exp", "map", "res", "rc", "tlb",
    "manifest", "cache", "tmp", "temp", "bak", "swp", "swo", "orig", "rej",
    
    // Java ecosystem
    "class", "jar", "war", "ear", "jmod", "jimage", "hprof", "heap", "jfr",
    
    // .NET ecosystem
    "pdb", "mdb", "nupkg", "snupkg", "vsix", "msi", "msm", "msp", "mst",
    "wixlib", "wixobj", "wixpdb", "cab", "msu", "exe", "dll", "winmd",
    
    // Protocol Buffers and serialization
    "pb", "protobuf", "proto", "avro", "parquet", "orc", "arrow", "cbor",
    "msgpack", "bson", "pickle", "pkl", "p", "joblib", "npy", "npz",
    
    // Virtual machines and containers
    "iso", "img", "vdi", "vmdk", "qcow2", "vhd", "vhdx", "hdd", "parallels",
    "pvm", "ova", "ovf", "box", "vagrant", "docker", "tar", "tgz",
    
    // Game and 3D files
    "pak", "wad", "vpk", "gcf", "ncf", "xnb", "unity3d", "unitypackage",
    "fbx", "dae", "obj", "3ds", "max", "blend", "ma", "mb", "c4d", "lwo",
    "lws", "mesh", "x3d", "ply", "stl", "off", "ase", "mdl", "md2", "md3",
    
    // CAD and engineering
    "dwg", "dxf", "step", "stp", "iges", "igs", "catpart", "catproduct",
    "prt", "asm", "par", "psm", "x_t", "x_b", "sat", "3dm", "rvt", "rfa",
    "ifc", "skp", "kmz", "kml",
    
    // Scientific and data files
    "hdf5", "h5", "nc", "cdf", "fits", "fts", "mat", "sav", "dta", "por",
    "sas7bdat", "xpt", "rda", "rds", "feather", "arrow", "zarr", "blosc",
    
    // Fonts
    "ttf", "otf", "woff", "woff2", "eot", "fon", "fnt", "pfb", "pfm", "afm",
    "bdf", "pcf", "snf", "gsf", "t1", "t42", "cff", "cid", "dfont", "suit",
    
    // Blockchain and crypto
    "dat", "blk", "rev", "wallet", "keystore", "p12", "pfx", "jks", "bks",
    
    // Mobile and embedded
    "apk", "ipa", "xap", "appx", "msix", "aab", "dex", "odex", "vdex",
    "art", "oat", "hex", "bin", "elf", "axf", "out", "uf2", "hex",
    
    // Others and miscellaneous
    "bin", "dat", "dump", "raw", "rom", "firmware", "bios", "efi", "uefi",
    "swap", "hiberfil", "pagefile", "core", "crashdump", "mdmp", "dmp",
    "lock", "pid", "sock", "fifo", "pipe", "device", "special",
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_binary_extension_detection() {
        let detector = BinaryDetector::default();
        
        // Test binary extensions
        assert!(detector.is_binary_by_extension(Path::new("test.exe")));
        assert!(detector.is_binary_by_extension(Path::new("test.jpg")));
        assert!(detector.is_binary_by_extension(Path::new("test.pdf")));
        assert!(detector.is_binary_by_extension(Path::new("TEST.EXE"))); // case insensitive
        
        // Test text extensions
        assert!(!detector.is_binary_by_extension(Path::new("test.txt")));
        assert!(!detector.is_binary_by_extension(Path::new("test.rs")));
        assert!(!detector.is_binary_by_extension(Path::new("test.py")));
        assert!(!detector.is_binary_by_extension(Path::new("Makefile")));
    }

    #[test]
    fn test_text_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create a text file
        let text_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&text_file)?;
        writeln!(file, "This is a text file with some content.")?;
        writeln!(file, "It has multiple lines and should be detected as text.")?;
        
        assert!(detector.is_text_file(&text_file)?);
        assert!(!detector.is_binary(&text_file)?);

        Ok(())
    }

    #[test] 
    fn test_binary_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create a binary file with null bytes
        let binary_file = temp_dir.path().join("test.bin");
        let mut file = File::create(&binary_file)?;
        file.write_all(&[0x00, 0x01, 0x02, 0x03, 0xFF, 0xFE, 0xFD])?;
        
        assert!(!detector.is_text_file(&binary_file)?);
        assert!(detector.is_binary(&binary_file)?);

        Ok(())
    }

    #[test]
    fn test_empty_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create an empty file
        let empty_file = temp_dir.path().join("empty.txt");
        File::create(&empty_file)?;
        
        // Empty files should be treated as text
        assert!(detector.is_text_file(&empty_file)?);
        assert!(!detector.is_binary(&empty_file)?);

        Ok(())
    }

    #[test]
    fn test_utf8_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create a UTF-8 file with unicode content
        let utf8_file = temp_dir.path().join("unicode.txt");
        let mut file = File::create(&utf8_file)?;
        writeln!(file, "Hello, 世界! 🌍")?;
        writeln!(file, "This file contains émojis and unicode characters: 日本語")?;
        drop(file);
        
        assert!(detector.is_text_file(&utf8_file)?);
        assert!(!detector.is_binary(&utf8_file)?);

        Ok(())
    }

    #[test]
    fn test_binary_reason() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Test extension-based detection
        let exe_file = temp_dir.path().join("test.exe");
        File::create(&exe_file)?;
        let reason = detector.get_binary_reason(&exe_file)?;
        assert!(reason.is_some());
        assert!(reason.unwrap().contains("extension"));

        // Test null byte detection
        let binary_file = temp_dir.path().join("test.bin"); // Use .bin extension to force binary detection first
        let mut file = File::create(&binary_file)?;
        file.write_all(b"some text\x00more text")?;
        drop(file); // Ensure file is closed
        let reason = detector.get_binary_reason(&binary_file)?;
        assert!(reason.is_some());
        let reason_str = reason.unwrap();
        assert!(reason_str.contains("extension") || reason_str.contains("null") || reason_str.contains("binary"));

        // Test text file (should not be binary)
        let text_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&text_file)?;
        writeln!(file, "This is just text")?;
        let reason = detector.get_binary_reason(&text_file)?;
        assert!(reason.is_none());

        Ok(())
    }

    #[test]
    fn test_printable_ascii() {
        // Test printable characters
        assert!(is_printable_ascii(b' '));  // space
        assert!(is_printable_ascii(b'A'));  // letter
        assert!(is_printable_ascii(b'0'));  // digit
        assert!(is_printable_ascii(b'~'));  // tilde
        assert!(is_printable_ascii(b'\t')); // tab
        assert!(is_printable_ascii(b'\n')); // newline
        assert!(is_printable_ascii(b'\r')); // carriage return

        // Test non-printable characters
        assert!(!is_printable_ascii(0x00)); // null
        assert!(!is_printable_ascii(0x01)); // control character
        assert!(!is_printable_ascii(0x7F)); // DEL
        assert!(!is_printable_ascii(0x80)); // extended ASCII
        assert!(!is_printable_ascii(0xFF)); // extended ASCII
    }

    #[test]
    fn test_utf8_start_detection() {
        // Valid UTF-8 start bytes
        assert!(is_valid_utf8_start(0x41));  // ASCII 'A'
        assert!(is_valid_utf8_start(0xC2));  // 2-byte UTF-8 start
        assert!(is_valid_utf8_start(0xE0));  // 3-byte UTF-8 start
        assert!(is_valid_utf8_start(0xF0));  // 4-byte UTF-8 start

        // Invalid UTF-8 start bytes
        assert!(!is_valid_utf8_start(0x80)); // continuation byte
        assert!(!is_valid_utf8_start(0xBF)); // continuation byte
        assert!(!is_valid_utf8_start(0xF8)); // invalid start byte
        assert!(!is_valid_utf8_start(0xFF)); // invalid start byte
    }

    #[test]
    fn test_protocol_buffer_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create a mock Protocol Buffer file
        // Protocol Buffer wire format: field_number << 3 | wire_type
        let protobuf_data = vec![
            0x08, 0x96, 0x01,       // field 1, varint, value 150
            0x12, 0x04, 0x74, 0x65, 0x73, 0x74, // field 2, string, "test"
            0x1a, 0x03, 0x66, 0x6f, 0x6f,       // field 3, string, "foo"
            0x20, 0x42,             // field 4, varint, value 66
        ];

        let protobuf_file = temp_dir.path().join("test.pb");
        let mut file = File::create(&protobuf_file)?;
        file.write_all(&protobuf_data)?;
        drop(file);

        // Should be detected as binary due to Protocol Buffer patterns
        assert!(detector.is_binary(&protobuf_file)?);
        assert!(!detector.is_text_file(&protobuf_file)?);

        // Test the reason detection
        let reason = detector.get_binary_reason(&protobuf_file)?;
        assert!(reason.is_some());
        let reason_str = reason.unwrap();
        assert!(reason_str.contains("Protocol Buffer") || reason_str.contains("binary") || reason_str.contains("extension"));

        Ok(())
    }

    #[test]
    fn test_protocol_buffer_patterns() {
        let detector = BinaryDetector::default();

        // Test typical Protocol Buffer patterns
        let protobuf_patterns = vec![
            // Varint field (field 1, wire type 0)
            0x08, 0x96, 0x01,
            // Length-delimited field (field 2, wire type 2) 
            0x12, 0x04, 0x74, 0x65, 0x73, 0x74,
            // Another varint
            0x20, 0x42,
        ];

        assert!(detector.is_protocol_buffer(&protobuf_patterns));

        // Test non-protobuf data
        let text_data = b"This is plain text content";
        assert!(!detector.is_protocol_buffer(text_data));

        // Test empty or very short data
        assert!(!detector.is_protocol_buffer(&[]));
        assert!(!detector.is_protocol_buffer(&[0x01]));
    }

    #[test]
    fn test_protobuf_vs_text_distinction() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create a file that might look like protobuf but is actually text
        let pseudo_protobuf_text = temp_dir.path().join("pseudo.txt");
        let mut file = File::create(&pseudo_protobuf_text)?;
        writeln!(file, "This file contains some bytes like \\x08 and \\x12 but is text")?;
        drop(file);

        // Should be detected as text, not protobuf
        assert!(detector.is_text_file(&pseudo_protobuf_text)?);
        assert!(!detector.is_binary(&pseudo_protobuf_text)?);

        // Create an actual binary protobuf-like file
        let real_protobuf = temp_dir.path().join("real.pb");
        let protobuf_data = vec![
            0x08, 0x96, 0x01,                   // field 1, varint
            0x12, 0x04, 0x74, 0x65, 0x73, 0x74, // field 2, "test"
            0x1a, 0x06, 0x62, 0x69, 0x6e, 0x61, 0x72, 0x79, // field 3, "binary"
            0x20, 0x2a,                         // field 4, varint
            0x2a, 0x08, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, // field 5, bytes
        ];
        let mut file = File::create(&real_protobuf)?;
        file.write_all(&protobuf_data)?;
        drop(file);

        // Should be detected as binary protobuf
        assert!(detector.is_binary(&real_protobuf)?);
        assert!(!detector.is_text_file(&real_protobuf)?);

        Ok(())
    }

    #[test]
    fn test_pb_file_detection() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create a .pb file (protobuf file)
        let pb_file = temp_dir.path().join("test.pb");
        let mut file = File::create(&pb_file)?;
        file.write_all(b"some protobuf binary data")?;
        drop(file);
        
        // .pb files should be detected as binary by extension
        assert!(detector.is_binary(&pb_file)?, ".pb file should be detected as binary");
        assert!(!detector.is_text_file(&pb_file)?, ".pb file should NOT be detected as text");
        
        let reason = detector.get_binary_reason(&pb_file)?;
        assert!(reason.is_some(), ".pb file should have a binary reason");
        assert!(reason.unwrap().contains("extension"), "Reason should mention extension");

        Ok(())
    }

    #[test]
    fn test_discovery_phase_simulation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let detector = BinaryDetector::default();

        // Create various file types to simulate a discovery phase
        let pb_file = temp_dir.path().join("test.pb");
        let txt_file = temp_dir.path().join("test.txt");
        let rs_file = temp_dir.path().join("test.rs");
        
        // Create .pb file with some content that might contain search string
        let mut file = File::create(&pb_file)?;
        file.write_all(b"oldstring should not be processed in binary files")?;
        drop(file);
        
        // Create text files
        let mut file = File::create(&txt_file)?;
        file.write_all(b"This contains oldstring and should be processed")?;
        drop(file);
        
        let mut file = File::create(&rs_file)?;
        file.write_all(b"fn main() { println!(\"oldstring\"); }")?;
        drop(file);
        
        // Simulate the file_needs_content_replacement logic
        let search_string = "oldstring";
        
        // Test .pb file - should NOT be added to content_files
        let pb_is_text = detector.is_text_file(&pb_file)?;
        println!("pb_file is_text_file: {}", pb_is_text);
        if pb_is_text {
            // This would be the bug - if we get here, .pb files are being misclassified
            let contains_string = std::fs::read_to_string(&pb_file)?.contains(search_string);
            println!("ERROR: .pb file classified as text and contains search string: {}", contains_string);
            assert!(false, ".pb file should not be classified as text!");
        } else {
            println!("OK: .pb file correctly classified as binary, will not be processed for content");
        }
        
        // Test text files - should be added to content_files if they contain the string
        assert!(detector.is_text_file(&txt_file)?, "txt file should be text");
        assert!(detector.is_text_file(&rs_file)?, "rs file should be text");
        
        Ok(())
    }
}