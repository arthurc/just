mod parser;

use std::{
    convert::TryFrom,
    fmt::{self, Debug},
};

use byteorder::NativeEndian;

pub use crate::JImageError;

use self::parser::Parser;

const HASH_MULTIPLIER: i32 = 0x01000193;

#[derive(PartialEq, Debug)]
pub enum AttributeKind {
    Module,
    Parent,
    Base,
    Extension,
    Offset,
    Compressed,
    Uncompressed,

    Total,
}

#[derive(Debug)]
pub struct Header {
    pub version: (u16, u16),
    pub flags: u32,
    pub resource_count: u32,
    pub table_length: u32,
    pub attributes_size: u32,
    pub strings_size: u32,
}
impl Header {
    pub fn index_size(&self) -> usize {
        std::mem::size_of::<u32>() // Magic identifier
            + std::mem::size_of::<Header>()
            + self.redirect_table_size()
            + self.attribute_offsets_size()
            + self.attributes_size as usize
            + self.strings_size as usize
    }

    pub fn redirect_table_size(&self) -> usize {
        self.table_length as usize * std::mem::size_of::<i32>()
    }

    pub fn attribute_offsets_size(&self) -> usize {
        self.table_length as usize * std::mem::size_of::<u32>()
    }
}

impl std::fmt::Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, " Major Version:  {}", self.version.0)?;
        writeln!(f, " Minor Version:  {}", self.version.1)?;
        writeln!(f, " Flags:          {}", self.flags)?;
        writeln!(f, " Resource Count: {}", self.resource_count)?;
        writeln!(f, " Table Length:   {}", self.table_length)?;
        writeln!(f, " Offsets Size:   {}", self.attribute_offsets_size())?;
        writeln!(f, " Redirects Size: {}", self.redirect_table_size())?;
        writeln!(f, " Locations Size: {}", self.attributes_size)?;
        writeln!(f, " Strings Size:   {}", self.strings_size)?;
        writeln!(f, " Index Size:     {}", self.index_size())?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Index {
    redirect_table: Vec<i32>,
    attribute_offsets: Vec<u32>,
    strings_data: Vec<u8>,
    attribute_data: Vec<u8>,
}

impl TryFrom<u8> for AttributeKind {
    type Error = u8;

    fn try_from(value: u8) -> std::result::Result<Self, Self::Error> {
        match value {
            1 => Ok(AttributeKind::Module),
            2 => Ok(AttributeKind::Parent),
            3 => Ok(AttributeKind::Base),
            4 => Ok(AttributeKind::Extension),
            5 => Ok(AttributeKind::Offset),
            6 => Ok(AttributeKind::Compressed),
            7 => Ok(AttributeKind::Uncompressed),
            _ => Err(value),
        }
    }
}

pub struct Archive<'a> {
    buf: &'a [u8],
    header: Header,
    index: Index,
    resource_data_start: usize,
}
impl<'a> Archive<'a> {
    pub fn parse(buf: &'a [u8]) -> Result<Self, JImageError> {
        Parser::<NativeEndian>::new(buf).parse_archive()
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn resources(&self) -> Resources {
        Resources {
            archive: self,
            index: 0,
        }
    }

    pub fn by_name(&self, path: &str) -> Option<Resource> {
        let hash_code = hash(path, HASH_MULTIPLIER);
        let index = hash_code % self.index.redirect_table.len() as i32;
        let value = self.index.redirect_table[index as usize];
        if value == 0 {
            return None;
        }
        let value = if value > 0 {
            hash(path, value) % self.index.redirect_table.len() as i32
        } else {
            -1 - value
        };

        let attributes_offset = self.index.attribute_offsets[value as usize];
        let attributes_data = &self.index.attribute_data[attributes_offset as usize..];

        let attributes = Parser::<NativeEndian>::new(attributes_data)
            .parse_attributes()
            .ok()?;

        let resource = Resource {
            archive: self,
            attributes,
        };

        if Self::verify(&resource, path) {
            Some(resource)
        } else {
            None
        }
    }

    fn verify(resource: &Resource, path: &str) -> bool {
        // Module
        let path = if resource.module().len() > 0 {
            if path.chars().nth(0) != Some('/')
                || !path[1..].starts_with(resource.module())
                || path.chars().nth(1 + resource.module().len()) != Some('/')
            {
                return false;
            }
            &path[2 + resource.module().len()..]
        } else {
            path
        };

        // Package
        let path = if resource.parent().len() > 0 {
            if !path.starts_with(resource.parent())
                || path.chars().nth(resource.parent().len()) != Some('/')
            {
                return false;
            }
            &path[1 + resource.parent().len()..]
        } else {
            path
        };

        // Basename
        let path = if !path.starts_with(resource.base()) {
            return false;
        } else {
            &path[resource.base().len()..]
        };

        // Extension
        let path = if resource.extension().len() > 0 {
            if path.chars().nth(0) != Some('.') || !path[1..].starts_with(resource.extension()) {
                return false;
            }
            &path[1 + resource.extension().len()..]
        } else {
            path
        };

        return path.len() == 0;
    }
}

fn hash(data: &str, seed: i32) -> i32 {
    let hash_code = data.bytes().into_iter().fold(seed as u32, |useed, byte| {
        (useed.wrapping_mul(HASH_MULTIPLIER as u32)) ^ byte as u32
    });
    return (hash_code & 0x7fff_ffff) as i32;
}

pub struct Resources<'a> {
    archive: &'a Archive<'a>,
    index: usize,
}
impl<'a> Iterator for Resources<'a> {
    type Item = Resource<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.archive.index.redirect_table.len() {
            return None;
        }

        let attribute_offset = self.archive.index.attribute_offsets[self.index];
        let attributes = Parser::<NativeEndian>::new(
            &self.archive.index.attribute_data[attribute_offset as usize..],
        )
        .parse_attributes()
        .unwrap();

        self.index += 1;

        Some(Resource {
            archive: self.archive,
            attributes,
        })
    }
}

pub struct Resource<'a> {
    attributes: [u64; AttributeKind::Total as usize],
    archive: &'a Archive<'a>,
}
impl fmt::Debug for Resource<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource")
            .field("attributes", &self.attributes)
            .finish()
    }
}
impl<'a> Resource<'a> {
    pub fn module(&self) -> &str {
        self.string_at(AttributeKind::Module)
    }

    pub fn parent(&self) -> &str {
        self.string_at(AttributeKind::Parent)
    }

    pub fn base(&self) -> &str {
        self.string_at(AttributeKind::Base)
    }

    pub fn extension(&self) -> &str {
        self.string_at(AttributeKind::Extension)
    }

    pub fn offset(&self) -> usize {
        self.attributes[AttributeKind::Offset as usize] as usize
    }

    pub fn bytes(&self) -> &'a [u8] {
        let offset = self.archive.resource_data_start + self.offset();
        let size = self.attributes[AttributeKind::Uncompressed as usize] as usize;
        &self.archive.buf[offset..offset + size]
    }

    pub fn full_name(&self) -> String {
        let mut s = String::with_capacity(10);

        if let Some(module) = self.try_string(AttributeKind::Module) {
            s.push_str("/");
            s.push_str(module);
            s.push_str("/");
        }

        if let Some(parent) = self.try_string(AttributeKind::Parent) {
            s.push_str(parent);
            s.push_str("/");
        }

        if let Some(base) = self.try_string(AttributeKind::Base) {
            s.push_str(base);
        }

        if let Some(extension) = self.try_string(AttributeKind::Extension) {
            s.push_str(".");
            s.push_str(extension);
        }

        s
    }

    fn string_at(&self, attribute_kind: AttributeKind) -> &str {
        self.try_string(attribute_kind).unwrap_or_default()
    }

    fn attribute_offset(&self, attribute_kind: AttributeKind) -> usize {
        self.attributes[attribute_kind as usize] as usize
    }

    fn try_string(&self, attribute_kind: AttributeKind) -> Option<&str> {
        let offset = self.attribute_offset(attribute_kind);
        let bytes = self.archive.index.strings_data[offset..]
            .split(|n| *n == 0)
            .next()?;

        if bytes.is_empty() {
            return None;
        }

        std::str::from_utf8(bytes).ok()
    }
}
