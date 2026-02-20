use arboard::Clipboard as ArboardClipboard;

/// Error type for clipboard operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardError {
    InitFailed(String),
    CopyFailed(String),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::InitFailed(msg) => {
                write!(f, "Clipboard initialization failed: {}", msg)
            }
            ClipboardError::CopyFailed(msg) => write!(f, "Clipboard copy failed: {}", msg),
        }
    }
}

impl std::error::Error for ClipboardError {}

/// Wrapper around arboard clipboard with error handling
pub struct Clipboard {
    inner: ArboardClipboard,
}

impl std::fmt::Debug for Clipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Clipboard")
            .field("inner", &"<arboard::Clipboard>")
            .finish()
    }
}

impl Clipboard {
    /// Initialize clipboard (may fail on headless systems)
    pub fn new() -> Result<Self, ClipboardError> {
        let inner =
            ArboardClipboard::new().map_err(|e| ClipboardError::InitFailed(e.to_string()))?;

        Ok(Self { inner })
    }

    /// Copy text to system clipboard
    pub fn copy(&mut self, text: &str) -> Result<(), ClipboardError> {
        self.inner
            .set_text(text)
            .map_err(|e| ClipboardError::CopyFailed(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_error_display() {
        let err = ClipboardError::InitFailed("no display".to_string());
        assert!(err.to_string().contains("initialization failed"));

        let err = ClipboardError::CopyFailed("access denied".to_string());
        assert!(err.to_string().contains("copy failed"));
    }
}
