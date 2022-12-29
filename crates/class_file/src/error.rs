use thiserror::Error;

use crate::constant_pool;

#[derive(Error, Debug)]
pub enum ClassFileError {
    #[error(transparent)]
    IOError(#[from] std::io::Error),
    #[error("Expected {0}, found {1:?}")]
    UnexpectedConstantPoolEntry(&'static str, constant_pool::CpInfo),
    #[error("Invalid magic identifier: 0x{0:X}")]
    InvalidCpInfoTag(u8),
    #[error("Invalid cp info tag: {0}")]
    InvalidMagicIdentifier(u32),
}
