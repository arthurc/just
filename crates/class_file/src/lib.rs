// https://docs.oracle.com/javase/specs/jvms/se19/html/jvms-4.html

mod access_flags;
pub mod attributes;
mod class_file;
#[macro_use]
mod constant_pool;
mod error;
mod parser;

use std::fmt;

pub use self::class_file::ClassFile;
pub use access_flags::AccessFlags;
pub use constant_pool::ConstantPool;
pub use error::ClassFileError;
pub use parser::Parser;

pub type Result<T, E = ClassFileError> = std::result::Result<T, E>;

pub struct Attribute {
    pub attribute_name_index: u16,
    pub info: Vec<u8>,
}
impl fmt::Debug for Attribute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Attribute")
            .field("attribute_name_index", &self.attribute_name_index)
            .field("info", &format!("({} bytes)", self.info.len()))
            .finish()
    }
}
