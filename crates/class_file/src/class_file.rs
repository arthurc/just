use std::io::{Read, Seek};

use crate::{
    attributes::Attributes, constant_pool::ClassInfo, matches_cp_info, parser::Parser, AccessFlags,
    ConstantPool, Result,
};

#[derive(Debug)]
pub struct ClassFile {
    pub constant_pool: ConstantPool,
    pub access_flags: AccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces: Vec<u16>,
    pub fields: Vec<FieldInfo>,
    pub methods: Vec<MethodInfo>,
    pub attributes: Attributes,
}
impl ClassFile {
    pub fn parse(bytes: impl Read + Seek) -> Result<ClassFile> {
        Ok(Parser::new(bytes).parse()?)
    }

    pub fn super_class(&self) -> Result<Option<&str>> {
        // For a class, the value of the super_class item either must be zero or must be a valid index
        // into the constant_pool table. If the value of the super_class item is nonzero, the
        // constant_pool entry at that index must be a CONSTANT_Class_info structure representing the
        // direct superclass of the class defined by this class file. Neither the direct superclass nor
        // any of its superclasses may have the ACC_FINAL flag set in the access_flags item of its
        // ClassFile structure.
        //
        // FIXME: For an interface, the value of the super_class item must always be a valid index
        //        into the constant_pool table. The constant_pool entry at that index must be a
        //        CONSTANT_Class_info structure representing the class Object.

        let ClassInfo { name_index } =
            matches_cp_info!(self.constant_pool, self.super_class, Class)?;

        // If the value of the super_class item is zero, then this class file must represent the class Object,
        // the only class or interface without a direct superclass.
        if *name_index == 0 {
            return Ok(None);
        }

        Ok(Some(matches_cp_info!(
            self.constant_pool,
            *name_index,
            Utf8
        )?))
    }

    pub fn class_name(&self) -> Result<&str> {
        // The value of the this_class item must be a valid index into the constant_pool table.
        // The constant_pool entry at that index must be a CONSTANT_Class_info structure (§4.4.1)
        // representing the class or interface defined by this class file.

        let ClassInfo { name_index } =
            matches_cp_info!(self.constant_pool, self.this_class, Class)?;

        matches_cp_info!(self.constant_pool, *name_index, Utf8)
    }

    pub fn field_name(&self, field: &FieldInfo) -> Result<&str> {
        Ok(matches_cp_info!(
            self.constant_pool,
            field.name_index,
            Utf8
        )?)
    }

    pub fn field_descriptor(&self, field: &FieldInfo) -> Result<&str> {
        Ok(matches_cp_info!(
            self.constant_pool,
            field.descriptor_index,
            Utf8
        )?)
    }

    pub fn method_name(&self, method: &MethodInfo) -> Result<&str> {
        Ok(matches_cp_info!(
            self.constant_pool,
            method.name_index,
            Utf8
        )?)
    }

    pub fn method_descriptor(&self, method: &MethodInfo) -> Result<&str> {
        Ok(matches_cp_info!(
            self.constant_pool,
            method.descriptor_index,
            Utf8
        )?)
    }
}

#[derive(Debug)]
pub struct FieldInfo {
    pub access_flags: AccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Attributes,
}

#[derive(Debug)]
pub struct MethodInfo {
    pub access_flags: AccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes: Attributes,
}
