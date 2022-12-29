use std::io::Cursor;

use crate::{constant_pool::CpInfo, Attribute};

use super::{parser::Parser, ConstantPool};

#[derive(Debug)]
pub struct Attributes(pub Vec<Attribute>);
impl Attributes {
    pub fn find_by_name(&self, name: &str, constant_pool: &ConstantPool) -> Option<&Attribute> {
        for a in &self.0 {
            let CpInfo::Utf8(ref s) = constant_pool[a.attribute_name_index] else {
                continue;
            };

            if s == name {
                return Some(a);
            }
        }

        None
    }

    pub fn code_attribute(&self, constant_pool: &ConstantPool) -> Option<CodeAttribute> {
        Parser::new(Cursor::new(&self.find_by_name("Code", constant_pool)?.info))
            .parse_code_attribute()
            .ok()
    }
}

#[derive(Debug)]
pub struct ExceptionTableEntry {
    pub start_pc: u16,
    pub end_pc: u16,
    pub handler_pc: u16,
    pub catch_type: u16,
}

#[derive(Debug)]
pub struct CodeAttribute {
    pub max_stack: u16,
    pub max_locals: u16,
    pub code: Vec<u8>,
    pub exception_table: Vec<ExceptionTableEntry>,
    pub attributes: Attributes,
}
