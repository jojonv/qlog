use std::borrow::Cow;

/// A zero-copy string view into a memory-mapped file.
/// Provides access to raw bytes and lossy UTF-8 conversion.
#[derive(Debug, Clone, Copy)]
pub struct MmapStr<'a> {
    data: &'a [u8],
}

impl<'a> MmapStr<'a> {
    /// Create a new MmapStr from a byte slice.
    pub fn new(data: &'a [u8]) -> Self {
        Self { data }
    }

    /// Get the underlying bytes.
    pub fn as_bytes(&self) -> &'a [u8] {
        self.data
    }

    /// Get a lossy UTF-8 string view.
    /// Invalid UTF-8 sequences are replaced with the Unicode replacement character.
    pub fn as_str_lossy(&self) -> Cow<'a, str> {
        String::from_utf8_lossy(self.data)
    }

    /// Get the byte length of the content.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the content is empty.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mmap_str_basic() {
        let data = b"Hello, World!";
        let mmap_str = MmapStr::new(data);

        assert_eq!(mmap_str.as_bytes(), data);
        assert_eq!(mmap_str.as_str_lossy(), "Hello, World!");
        assert_eq!(mmap_str.len(), 13);
        assert!(!mmap_str.is_empty());
    }

    #[test]
    fn test_mmap_str_lossy_utf8() {
        // Invalid UTF-8 bytes - each produces 1 replacement char (3 bytes in UTF-8)
        let data = vec![0x80, 0x81, 0x82];
        let mmap_str = MmapStr::new(&data);

        // Should not panic, should return replacement characters
        // Each replacement char is 3 bytes in UTF-8, so 3 * 3 = 9 bytes total
        let lossy = mmap_str.as_str_lossy();
        assert_eq!(lossy.len(), 9); // 3 replacement chars * 3 bytes each
    }

    #[test]
    fn test_mmap_str_empty() {
        let data = b"";
        let mmap_str = MmapStr::new(data);

        assert!(mmap_str.is_empty());
        assert_eq!(mmap_str.len(), 0);
    }
}
