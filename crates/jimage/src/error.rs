use thiserror::Error;

#[derive(Debug, Error)]
pub enum JImageError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Invalid attribute kind: {0}")]
    InvalidAttributeKind(u8),
    #[error("Invalid magic identifier: 0x{0:X}")]
    InvalidMagicIdentifier(u32),
}
