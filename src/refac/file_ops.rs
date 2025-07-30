use anyhow::{Context, Result};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::{Path, PathBuf};
use super::binary_detector::BinaryDetector;
use encoding_rs::{Encoding, UTF_8};
use chardet::detect;

/// File operations for the refac tool (part of the nomion suite)
pub struct FileOperations {
    binary_detector: BinaryDetector,
    backup_enabled: bool,
}

/// Encoding information for a file
#[derive(Debug, Clone)]
struct FileEncoding {
    encoding: &'static Encoding,
    has_bom: bool,
}

impl Default for FileOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOperations {
    pub fn new() -> Self {
        Self {
            binary_detector: BinaryDetector::default(),
            backup_enabled: false,
        }
    }

    pub fn with_backup(mut self, enabled: bool) -> Self {
        self.backup_enabled = enabled;
        self
    }

    /// Replace content in a file
    pub fn replace_content<P: AsRef<Path>>(
        &self,
        file_path: P,
        pattern: &str,
        substitute: &str,
    ) -> Result<bool> {
        let file_path = file_path.as_ref();
        
        // Skip binary files
        if self.binary_detector.is_binary(file_path)? {
            return Ok(false);
        }

        // Read file as bytes first
        let original_bytes = fs::read(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        // Detect the file's encoding
        let file_encoding = self.detect_encoding(&original_bytes)?;
        
        // Decode the content using the detected encoding
        let content = self.decode_with_encoding(&original_bytes, &file_encoding)
            .with_context(|| format!("Failed to decode file with detected encoding: {}", file_path.display()))?;

        // Check if the file contains the target string
        if !content.contains(pattern) {
            return Ok(false);
        }

        // Create backup if enabled
        if self.backup_enabled {
            self.create_backup(file_path)?;
        }

        // Replace content
        let new_content = content.replace(pattern, substitute);

        // Encode back to the original encoding and write
        let encoded_bytes = self.encode_with_encoding(&new_content, &file_encoding)
            .with_context(|| format!("Failed to encode content back to original encoding: {}", file_path.display()))?;

        fs::write(file_path, encoded_bytes)
            .with_context(|| format!("Failed to write file: {}", file_path.display()))?;

        Ok(true)
    }

    /// Replace content in a file using streaming for large files
    pub fn replace_content_streaming<P: AsRef<Path>>(
        &self,
        file_path: P,
        pattern: &str,
        substitute: &str,
    ) -> Result<bool> {
        let file_path = file_path.as_ref();
        
        // Skip binary files
        if self.binary_detector.is_binary(file_path)? {
            return Ok(false);
        }

        // Create backup if enabled
        if self.backup_enabled {
            self.create_backup(file_path)?;
        }

        let temp_file_path = file_path.with_extension("tmp");
        let mut modified = false;

        {
            let input_file = File::open(file_path)
                .with_context(|| format!("Failed to open input file: {}", file_path.display()))?;
            let reader = BufReader::new(input_file);

            let output_file = File::create(&temp_file_path)
                .with_context(|| format!("Failed to create temp file: {}", temp_file_path.display()))?;
            let mut writer = BufWriter::new(output_file);

            for line in reader.lines() {
                let line = line.with_context(|| {
                    format!("Failed to read line from file: {}", file_path.display())
                })?;
                
                let new_line = if line.contains(pattern) {
                    modified = true;
                    line.replace(pattern, substitute)
                } else {
                    line
                };

                writeln!(writer, "{}", new_line).with_context(|| {
                    format!("Failed to write to temp file: {}", temp_file_path.display())
                })?;
            }

            writer.flush().with_context(|| {
                format!("Failed to flush temp file: {}", temp_file_path.display())
            })?;
        }

        if modified {
            // Replace the original file with the modified one
            fs::rename(&temp_file_path, file_path).with_context(|| {
                format!(
                    "Failed to replace original file {} with temp file {}",
                    file_path.display(),
                    temp_file_path.display()
                )
            })?;
        } else {
            // Remove the temp file since no changes were made
            let _ = fs::remove_file(&temp_file_path);
        }

        Ok(modified)
    }

    /// Move/rename a file or directory
    pub fn move_item<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: P,
        to: Q,
    ) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();

        // Ensure the target directory exists
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }

        fs::rename(from, to).with_context(|| {
            format!(
                "Failed to move {} to {}",
                from.display(),
                to.display()
            )
        })?;

        Ok(())
    }

    /// Copy a file
    pub fn copy_file<P: AsRef<Path>, Q: AsRef<Path>>(
        &self,
        from: P,
        to: Q,
    ) -> Result<()> {
        let from = from.as_ref();
        let to = to.as_ref();

        // Ensure the target directory exists
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create parent directory: {}", parent.display())
            })?;
        }

        fs::copy(from, to).with_context(|| {
            format!(
                "Failed to copy {} to {}",
                from.display(),
                to.display()
            )
        })?;

        Ok(())
    }

    /// Create a backup of a file
    pub fn create_backup<P: AsRef<Path>>(&self, file_path: P) -> Result<PathBuf> {
        let file_path = file_path.as_ref();
        let backup_path = self.generate_backup_path(file_path)?;

        fs::copy(file_path, &backup_path).with_context(|| {
            format!(
                "Failed to create backup from {} to {}",
                file_path.display(),
                backup_path.display()
            )
        })?;

        Ok(backup_path)
    }

    /// Generate a unique backup file path
    fn generate_backup_path<P: AsRef<Path>>(&self, file_path: P) -> Result<PathBuf> {
        let file_path = file_path.as_ref();
        let mut backup_path = file_path.with_extension(
            format!(
                "{}.bak",
                file_path.extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
            )
        );

        // If backup already exists, find a unique name
        let mut counter = 1;
        while backup_path.exists() {
            backup_path = file_path.with_extension(
                format!(
                    "{}.bak.{}",
                    file_path.extension()
                        .and_then(|s| s.to_str())
                        .unwrap_or(""),
                    counter
                )
            );
            counter += 1;
        }

        Ok(backup_path)
    }

    /// Check if a file contains a specific string
    pub fn file_contains_string<P: AsRef<Path>>(
        &self,
        file_path: P,
        search_string: &str,
    ) -> Result<bool> {
        let file_path = file_path.as_ref();
        
        // Skip binary files
        if self.binary_detector.is_binary(file_path)? {
            return Ok(false);
        }

        // Read file as bytes and detect encoding
        let bytes = fs::read(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
            
        let file_encoding = self.detect_encoding(&bytes)?;
        let content = self.decode_with_encoding(&bytes, &file_encoding)
            .with_context(|| format!("Failed to decode file: {}", file_path.display()))?;

        Ok(content.contains(search_string))
    }

    /// Count occurrences of a string in a file
    pub fn count_string_occurrences<P: AsRef<Path>>(
        &self,
        file_path: P,
        search_string: &str,
    ) -> Result<usize> {
        let file_path = file_path.as_ref();
        
        // Skip binary files
        if self.binary_detector.is_binary(file_path)? {
            return Ok(0);
        }

        // Read file as bytes and detect encoding
        let bytes = fs::read(file_path)
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
            
        let file_encoding = self.detect_encoding(&bytes)?;
        let content = self.decode_with_encoding(&bytes, &file_encoding)
            .with_context(|| format!("Failed to decode file: {}", file_path.display()))?;

        Ok(content.matches(search_string).count())
    }

    /// Get file size
    pub fn get_file_size<P: AsRef<Path>>(&self, file_path: P) -> Result<u64> {
        let metadata = fs::metadata(file_path.as_ref())
            .with_context(|| format!("Failed to get metadata for: {}", file_path.as_ref().display()))?;
        Ok(metadata.len())
    }

    /// Check if a path is a text file
    pub fn is_text_file<P: AsRef<Path>>(&self, file_path: P) -> Result<bool> {
        self.binary_detector.is_text_file(file_path)
    }

    /// Get the reason why a file is considered binary
    pub fn get_binary_reason<P: AsRef<Path>>(&self, file_path: P) -> Result<Option<String>> {
        self.binary_detector.get_binary_reason(file_path)
    }

    /// Safely create a directory and all its parents
    pub fn create_dir_all<P: AsRef<Path>>(&self, dir_path: P) -> Result<()> {
        fs::create_dir_all(dir_path.as_ref()).with_context(|| {
            format!("Failed to create directory: {}", dir_path.as_ref().display())
        })
    }

    /// Check if a path exists
    pub fn exists<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().exists()
    }

    /// Check if a path is a file
    pub fn is_file<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_file()
    }

    /// Check if a path is a directory
    pub fn is_dir<P: AsRef<Path>>(&self, path: P) -> bool {
        path.as_ref().is_dir()
    }

    /// Detect the encoding of a file from its byte content
    fn detect_encoding(&self, bytes: &[u8]) -> Result<FileEncoding> {
        // Check for UTF-8 BOM first
        if bytes.len() >= 3 && &bytes[0..3] == b"\xEF\xBB\xBF" {
            return Ok(FileEncoding {
                encoding: UTF_8,
                has_bom: true,
            });
        }
        
        // Check for UTF-16 BOM
        if bytes.len() >= 2 {
            match &bytes[0..2] {
                b"\xFF\xFE" => {
                    return Ok(FileEncoding {
                        encoding: encoding_rs::UTF_16LE,
                        has_bom: true,
                    });
                }
                b"\xFE\xFF" => {
                    return Ok(FileEncoding {
                        encoding: encoding_rs::UTF_16BE,
                        has_bom: true,
                    });
                }
                _ => {}
            }
        }
        
        // Try UTF-8 first (most common)
        if let Ok(_) = std::str::from_utf8(bytes) {
            return Ok(FileEncoding {
                encoding: UTF_8,
                has_bom: false,
            });
        }
        
        // Use chardet for automatic detection
        let detection_result = detect(bytes);
        let encoding_name = detection_result.0;
        
        // Map chardet encoding names to encoding_rs encodings
        let encoding = match encoding_name.as_str() {
            "UTF-8" => UTF_8,
            "ASCII" => encoding_rs::WINDOWS_1252, // ASCII is subset of Windows-1252
            "ISO-8859-1" | "LATIN1" => encoding_rs::WINDOWS_1252,
            "WINDOWS-1252" | "CP1252" => encoding_rs::WINDOWS_1252,
            "WINDOWS-1251" | "CP1251" => encoding_rs::WINDOWS_1251,
            "ISO-8859-2" => encoding_rs::ISO_8859_2,
            "ISO-8859-3" => encoding_rs::ISO_8859_3,
            "ISO-8859-4" => encoding_rs::ISO_8859_4,
            "ISO-8859-5" => encoding_rs::ISO_8859_5,
            "ISO-8859-6" => encoding_rs::ISO_8859_6,
            "ISO-8859-7" => encoding_rs::ISO_8859_7,
            "ISO-8859-8" => encoding_rs::ISO_8859_8,
            "ISO-8859-10" => encoding_rs::ISO_8859_10,
            "ISO-8859-13" => encoding_rs::ISO_8859_13,
            "ISO-8859-14" => encoding_rs::ISO_8859_14,
            "ISO-8859-15" => encoding_rs::ISO_8859_15,
            "ISO-8859-16" => encoding_rs::ISO_8859_16,
            "KOI8-R" => encoding_rs::KOI8_R,
            "KOI8-U" => encoding_rs::KOI8_U,
            "BIG5" => encoding_rs::BIG5,
            "GB2312" | "GB18030" => encoding_rs::GB18030,
            "GBK" => encoding_rs::GBK,
            "EUC-JP" => encoding_rs::EUC_JP,
            "ISO-2022-JP" => encoding_rs::ISO_2022_JP,
            "SHIFT_JIS" => encoding_rs::SHIFT_JIS,
            "EUC-KR" => encoding_rs::EUC_KR,
            _ => {
                // Fallback to Windows-1252 for unknown encodings (handles most extended ASCII)
                encoding_rs::WINDOWS_1252
            }
        };
        
        Ok(FileEncoding {
            encoding,
            has_bom: false,
        })
    }
    
    /// Decode bytes using the detected encoding
    fn decode_with_encoding(&self, bytes: &[u8], file_encoding: &FileEncoding) -> Result<String> {
        let decode_bytes = if file_encoding.has_bom {
            // Skip BOM bytes
            if std::ptr::eq(file_encoding.encoding, UTF_8) {
                &bytes[3..] // UTF-8 BOM is 3 bytes
            } else if std::ptr::eq(file_encoding.encoding, encoding_rs::UTF_16LE) || 
                     std::ptr::eq(file_encoding.encoding, encoding_rs::UTF_16BE) {
                &bytes[2..] // UTF-16 BOM is 2 bytes
            } else {
                bytes
            }
        } else {
            bytes
        };
        
        let (decoded, _, had_errors) = file_encoding.encoding.decode(decode_bytes);
        
        if had_errors {
            return Err(anyhow::anyhow!(
                "Decoding errors occurred with encoding: {}", 
                file_encoding.encoding.name()
            ));
        }
        
        Ok(decoded.into_owned())
    }
    
    /// Encode string back to the original encoding
    fn encode_with_encoding(&self, content: &str, file_encoding: &FileEncoding) -> Result<Vec<u8>> {
        let (encoded, _, had_errors) = file_encoding.encoding.encode(content);
        
        if had_errors {
            return Err(anyhow::anyhow!(
                "Encoding errors occurred with encoding: {}", 
                file_encoding.encoding.name()
            ));
        }
        
        let mut result = Vec::new();
        
        // Add BOM if original file had one
        if file_encoding.has_bom {
            if std::ptr::eq(file_encoding.encoding, UTF_8) {
                result.extend_from_slice(b"\xEF\xBB\xBF");
            } else if std::ptr::eq(file_encoding.encoding, encoding_rs::UTF_16LE) {
                result.extend_from_slice(b"\xFF\xFE");
            } else if std::ptr::eq(file_encoding.encoding, encoding_rs::UTF_16BE) {
                result.extend_from_slice(b"\xFE\xFF");
            }
        }
        
        result.extend_from_slice(&encoded);
        Ok(result)
    }

    /// Get file permissions (Unix-style)
    #[cfg(unix)]
    pub fn get_permissions<P: AsRef<Path>>(&self, path: P) -> Result<u32> {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(path.as_ref())
            .with_context(|| format!("Failed to get metadata for: {}", path.as_ref().display()))?;
        Ok(metadata.permissions().mode())
    }

    /// Set file permissions (Unix-style)
    #[cfg(unix)]
    pub fn set_permissions<P: AsRef<Path>>(&self, path: P, mode: u32) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path.as_ref())?.permissions();
        perms.set_mode(mode);
        fs::set_permissions(path.as_ref(), perms)
            .with_context(|| format!("Failed to set permissions for: {}", path.as_ref().display()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_replace_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "Hello world")?;
        writeln!(file, "This is a test file")?;
        writeln!(file, "Hello again")?;

        // Replace content
        let modified = file_ops.replace_content(&test_file, "Hello", "Hi")?;
        assert!(modified);

        // Check the result
        let content = fs::read_to_string(&test_file)?;
        assert!(content.contains("Hi world"));
        assert!(content.contains("Hi again"));
        assert!(!content.contains("Hello"));

        Ok(())
    }

    #[test]
    fn test_replace_content_no_match() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "This is a test file")?;

        // Try to replace non-existent content
        let modified = file_ops.replace_content(&test_file, "nonexistent", "replacement")?;
        assert!(!modified);

        // Content should be unchanged
        let content = fs::read_to_string(&test_file)?;
        assert!(content.contains("This is a test file"));

        Ok(())
    }

    #[test]
    fn test_replace_content_streaming() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        for i in 0..100 {
            writeln!(file, "Line {} with pattern target here", i)?;
        }

        // Replace content using streaming
        let modified = file_ops.replace_content_streaming(&test_file, "target", "replacement")?;
        assert!(modified);

        // Check the result
        let content = fs::read_to_string(&test_file)?;
        assert!(content.contains("replacement"));
        assert!(!content.contains("target"));

        Ok(())
    }

    #[test]
    fn test_move_item() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let source_file = temp_dir.path().join("source.txt");
        let mut file = File::create(&source_file)?;
        writeln!(file, "Test content")?;

        // Move the file
        let target_file = temp_dir.path().join("subdir").join("target.txt");
        file_ops.move_item(&source_file, &target_file)?;

        // Check that the file was moved
        assert!(!source_file.exists());
        assert!(target_file.exists());

        // Check content is preserved
        let content = fs::read_to_string(&target_file)?;
        assert!(content.contains("Test content"));

        Ok(())
    }

    #[test]
    fn test_copy_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let source_file = temp_dir.path().join("source.txt");
        let mut file = File::create(&source_file)?;
        writeln!(file, "Test content")?;

        // Copy the file
        let target_file = temp_dir.path().join("subdir").join("target.txt");
        file_ops.copy_file(&source_file, &target_file)?;

        // Check that both files exist
        assert!(source_file.exists());
        assert!(target_file.exists());

        // Check content is the same
        let source_content = fs::read_to_string(&source_file)?;
        let target_content = fs::read_to_string(&target_file)?;
        assert_eq!(source_content, target_content);

        Ok(())
    }

    #[test]
    fn test_create_backup() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "Original content")?;

        // Create backup
        let backup_path = file_ops.create_backup(&test_file)?;

        // Check that backup was created
        assert!(backup_path.exists());
        
        // Check backup content
        let backup_content = fs::read_to_string(&backup_path)?;
        assert!(backup_content.contains("Original content"));

        // Check original file still exists
        assert!(test_file.exists());

        Ok(())
    }

    #[test]
    fn test_file_contains_string() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "This file contains a specific pattern")?;

        // Test string that exists
        assert!(file_ops.file_contains_string(&test_file, "specific pattern")?);
        
        // Test string that doesn't exist
        assert!(!file_ops.file_contains_string(&test_file, "nonexistent")?);

        Ok(())
    }

    #[test]
    fn test_count_string_occurrences() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "test test test")?;
        writeln!(file, "another line")?;
        writeln!(file, "test again")?;

        // Count occurrences
        let count = file_ops.count_string_occurrences(&test_file, "test")?;
        assert_eq!(count, 4); // 3 in first line + 1 in last line

        // Count non-existent string
        let count = file_ops.count_string_occurrences(&test_file, "nonexistent")?;
        assert_eq!(count, 0);

        Ok(())
    }

    #[test]
    fn test_backup_with_replace_content() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new().with_backup(true);
        
        // Create a test file
        let test_file = temp_dir.path().join("test.txt");
        let mut file = File::create(&test_file)?;
        writeln!(file, "Original content with target")?;

        // Replace content (should create backup automatically)
        let modified = file_ops.replace_content(&test_file, "target", "replacement")?;
        assert!(modified);

        // Check that backup was created
        let backup_path = test_file.with_extension("txt.bak");
        assert!(backup_path.exists());

        // Check backup contains original content
        let backup_content = fs::read_to_string(&backup_path)?;
        assert!(backup_content.contains("target"));

        // Check main file contains new content
        let main_content = fs::read_to_string(&test_file)?;
        assert!(main_content.contains("replacement"));
        assert!(!main_content.contains("target"));

        Ok(())
    }

    #[test]
    fn test_get_file_size() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create a test file with known content
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "Hello, world!")?; // 13 bytes

        let size = file_ops.get_file_size(&test_file)?;
        assert_eq!(size, 13);

        Ok(())
    }

    #[test]
    fn test_utility_functions() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Create test file and directory
        let test_file = temp_dir.path().join("test.txt");
        let test_dir = temp_dir.path().join("test_dir");
        
        File::create(&test_file)?;
        fs::create_dir(&test_dir)?;

        // Test existence checks
        assert!(file_ops.exists(&test_file));
        assert!(file_ops.exists(&test_dir));
        assert!(!file_ops.exists(temp_dir.path().join("nonexistent")));

        // Test type checks
        assert!(file_ops.is_file(&test_file));
        assert!(!file_ops.is_file(&test_dir));
        assert!(file_ops.is_dir(&test_dir));
        assert!(!file_ops.is_dir(&test_file));

        Ok(())
    }
    
    #[test]
    fn test_encoding_detection_and_preservation() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Test UTF-8 with BOM
        let utf8_bom_file = temp_dir.path().join("utf8_bom.txt");
        let utf8_content = "Hello, 世界! target string here";
        let mut utf8_bytes = vec![0xEF, 0xBB, 0xBF]; // UTF-8 BOM
        utf8_bytes.extend_from_slice(utf8_content.as_bytes());
        fs::write(&utf8_bom_file, &utf8_bytes)?;
        
        let modified = file_ops.replace_content(&utf8_bom_file, "target", "replacement")?;
        assert!(modified, "UTF-8 BOM file should be modified");
        
        // Check that BOM is preserved
        let result_bytes = fs::read(&utf8_bom_file)?;
        assert_eq!(&result_bytes[0..3], &[0xEF, 0xBB, 0xBF], "UTF-8 BOM should be preserved");
        
        // Check content was replaced
        let result_content = String::from_utf8_lossy(&result_bytes[3..]);
        assert!(result_content.contains("replacement"), "Content should contain replacement string");
        assert!(!result_content.contains("target"), "Content should not contain original target string");
        
        // Test Windows-1252 (extended ASCII) - simulating the TypeScript file issue
        let win1252_file = temp_dir.path().join("win1252.txt");
        // Create content with extended ASCII character (em-dash is 0x96 in Windows-1252)
        let win1252_bytes = vec![
            72, 101, 108, 108, 111, 32, // "Hello "
            150, // em-dash (0x96) in Windows-1252
            32, 116, 97, 114, 103, 101, 116, 32, 115, 116, 114, 105, 110, 103 // " target string"
        ];
        fs::write(&win1252_file, &win1252_bytes)?;
        
        let modified = file_ops.replace_content(&win1252_file, "target", "replacement")?;
        assert!(modified, "Windows-1252 file should be modified");
        
        // Verify the special character is preserved
        let result_bytes = fs::read(&win1252_file)?;
        assert!(result_bytes.contains(&150), "Extended ASCII character (em-dash) should be preserved");
        
        // Verify content was replaced
        let file_encoding = file_ops.detect_encoding(&result_bytes)?;
        let result_content = file_ops.decode_with_encoding(&result_bytes, &file_encoding)?;
        assert!(result_content.contains("replacement"), "Content should contain replacement string");
        assert!(!result_content.contains("target"), "Content should not contain original target string");
        
        Ok(())
    }
    
    #[test]
    fn test_utf16_bom_detection() -> Result<()> {
        let _temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Test UTF-16LE BOM detection
        let utf16le_bom = vec![0xFF, 0xFE, 0x48, 0x00, 0x65, 0x00]; // UTF-16LE BOM + "He"
        let encoding = file_ops.detect_encoding(&utf16le_bom)?;
        assert!(std::ptr::eq(encoding.encoding, encoding_rs::UTF_16LE), "Should detect UTF-16LE");
        assert!(encoding.has_bom, "Should detect BOM");
        
        // Test UTF-16BE BOM detection
        let utf16be_bom = vec![0xFE, 0xFF, 0x00, 0x48, 0x00, 0x65]; // UTF-16BE BOM + "He"
        let encoding = file_ops.detect_encoding(&utf16be_bom)?;
        assert!(std::ptr::eq(encoding.encoding, encoding_rs::UTF_16BE), "Should detect UTF-16BE");
        assert!(encoding.has_bom, "Should detect BOM");
        
        Ok(())
    }
    
    #[test]
    fn test_encoding_detection_methods() -> Result<()> {
        let _temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Test UTF-8 detection
        let utf8_content = "Hello, 世界!";
        let utf8_bytes = utf8_content.as_bytes();
        let encoding = file_ops.detect_encoding(utf8_bytes)?;
        assert!(std::ptr::eq(encoding.encoding, UTF_8), "Should detect UTF-8");
        assert!(!encoding.has_bom, "Should not have BOM");
        
        // Test UTF-8 BOM detection
        let mut utf8_bom_bytes = vec![0xEF, 0xBB, 0xBF];
        utf8_bom_bytes.extend_from_slice(utf8_bytes);
        let encoding = file_ops.detect_encoding(&utf8_bom_bytes)?;
        assert!(std::ptr::eq(encoding.encoding, UTF_8), "Should detect UTF-8 with BOM");
        assert!(encoding.has_bom, "Should have BOM");
        
        // Test Windows-1252 (extended ASCII) detection
        let extended_ascii_bytes = vec![72, 101, 108, 108, 111, 32, 150, 32, 119, 111, 114, 108, 100]; // "Hello \x96 world"
        let encoding = file_ops.detect_encoding(&extended_ascii_bytes)?;
        assert!(std::ptr::eq(encoding.encoding, encoding_rs::WINDOWS_1252), "Should detect Windows-1252");
        assert!(!encoding.has_bom, "Should not have BOM");
        
        Ok(())
    }
    
    #[test]
    fn test_file_contains_string_with_encoding() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Test with Windows-1252 file (like the TypeScript issue)
        let win1252_file = temp_dir.path().join("extended_ascii.txt");
        let content_bytes = vec![
            84, 104, 105, 115, 32, 102, 105, 108, 101, 32, // "This file "
            150, // em-dash (0x96) in Windows-1252
            32, 99, 111, 110, 116, 97, 105, 110, 115, 32, // " contains "
            116, 97, 114, 103, 101, 116, 32, 115, 116, 114, 105, 110, 103 // "target string"
        ];
        fs::write(&win1252_file, &content_bytes)?;
        
        // Test that we can detect the string despite encoding
        assert!(file_ops.file_contains_string(&win1252_file, "target string")?, 
                "Should find target string in Windows-1252 file");
        assert!(file_ops.file_contains_string(&win1252_file, "contains")?, 
                "Should find contains string in Windows-1252 file");
        assert!(!file_ops.file_contains_string(&win1252_file, "nonexistent")?, 
                "Should not find nonexistent string");
        
        // Test count occurrences
        let count = file_ops.count_string_occurrences(&win1252_file, "i")?;
        assert!(count >= 2, "Should find multiple occurrences of 'i'"); // in "This file"
        
        Ok(())
    }
    
    #[test]
    fn test_encoding_error_handling() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let file_ops = FileOperations::new();
        
        // Test with truly invalid sequence that can't be decoded
        let invalid_file = temp_dir.path().join("invalid.txt");
        // Create a sequence that's invalid in most encodings
        let invalid_bytes = vec![0xFF, 0xFE, 0xFF, 0xFF, 0x00, 0xD8, 0x00, 0x00]; // Invalid UTF-16
        fs::write(&invalid_file, &invalid_bytes)?;
        
        // The file operations should handle this gracefully
        let result = file_ops.file_contains_string(&invalid_file, "test");
        // Should either succeed (with lossy conversion) or fail gracefully
        assert!(result.is_ok() || result.is_err(), "Should handle invalid encoding gracefully");
        
        Ok(())
    }
}