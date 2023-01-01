use std::io::{BufReader, Read, Seek};

use byteorder::{BigEndian, ReadBytesExt};

use crate::{
    attributes::{Attributes, CodeAttribute, ExceptionTableEntry},
    class_file::{FieldInfo, MethodInfo},
};

use super::{constant_pool::CpInfo, *};

type Result<T, E = ClassFileError> = std::result::Result<T, E>;
type Endian = BigEndian;

pub struct Parser<R> {
    r: BufReader<R>,
}
impl<R: Read + Seek> Parser<R> {
    pub fn new(r: R) -> Self {
        Self {
            r: BufReader::new(r),
        }
    }

    pub fn parse(&mut self) -> Result<ClassFile> {
        let _ = self.parse_magic_identifier()?;
        let _version = self.parse_version()?;

        let constant_pool = self.parse_constant_pool()?;
        let access_flags = AccessFlags::from_bits_truncate(self.read_u16()?);
        let this_class = self.read_u16()?;
        let super_class = self.read_u16()?;
        let interfaces_count = self.read_u16()?;

        let mut interfaces = vec![0u16; interfaces_count as usize];
        self.r.read_u16_into::<Endian>(&mut interfaces)?;

        let fields_count = self.read_u16()?;
        let fields = (0..fields_count)
            .map(|_| self.parse_field_info())
            .collect::<Result<Vec<_>>>()?;

        let methods_count = self.read_u16()?;
        let methods = (0..methods_count)
            .map(|_| self.parse_method_info())
            .collect::<Result<Vec<_>>>()?;

        let attributes_count = self.read_u16()?;
        let attributes = self.parse_attributes(attributes_count)?;

        Ok(ClassFile {
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces,
            fields,
            methods,
            attributes,
        })
    }

    fn parse_field_info(&mut self) -> Result<FieldInfo> {
        let access_flags = AccessFlags::from_bits_truncate(self.read_u16()?);
        let name_index = self.read_u16()?;
        let descriptor_index = self.read_u16()?;
        let attributes_count = self.read_u16()?;
        let attributes = self.parse_attributes(attributes_count)?;

        Ok(FieldInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }

    fn parse_method_info(&mut self) -> Result<MethodInfo> {
        let access_flags = AccessFlags::from_bits_truncate(self.read_u16()?);
        let name_index = self.read_u16()?;
        let descriptor_index = self.read_u16()?;
        let attributes_count = self.read_u16()?;
        let attributes = self.parse_attributes(attributes_count)?;

        Ok(MethodInfo {
            access_flags,
            name_index,
            descriptor_index,
            attributes,
        })
    }

    fn parse_magic_identifier(&mut self) -> Result<()> {
        match self.read_u32()? {
            0xCAFEBABE => Ok(()),
            magic_identifier => Err(ClassFileError::InvalidMagicIdentifier(magic_identifier)),
        }
    }

    fn parse_version(&mut self) -> Result<(u16, u16)> {
        let minor = self.read_u16()?;
        let major = self.read_u16()?;
        Ok((major, minor))
    }

    fn parse_constant_pool(&mut self) -> Result<ConstantPool> {
        let constant_pool_count = self.read_u16()?;

        let mut count = constant_pool_count as usize - 1;
        let mut res = Vec::with_capacity(count);
        while count > 0 {
            let (cp_info, slot_size) = self.parse_cp_info()?;
            res.push(cp_info);
            (0..slot_size - 1).for_each(|_| res.push(CpInfo::Unusable));

            count -= slot_size;
        }
        Ok(ConstantPool::new(res))
    }

    fn parse_cp_info(&mut self) -> Result<(CpInfo, usize)> {
        let tag = self.read_u8()?;
        let (cp_info, additional_cp_info) = match tag {
            1 => (self.parse_utf8()?, 1),
            3 => (self.parse_integer()?, 1),
            4 => (self.parse_float()?, 1),
            5 => (self.parse_long()?, 2),
            7 => (self.parse_class_info()?, 1),
            8 => (self.parse_string()?, 1),
            9 => (self.parse_field_ref()?, 1),
            10 => (self.parse_method_ref()?, 1),
            11 => (self.parse_interface_method_ref()?, 1),
            12 => (self.parse_name_and_type_info()?, 1),
            15 => (self.parse_method_handle()?, 1),
            16 => (self.parse_method_type_info()?, 1),
            18 => (self.parse_invoke_dynamic_info()?, 1),
            _ => return Err(ClassFileError::InvalidCpInfoTag(tag)),
        };

        Ok((cp_info, additional_cp_info))
    }

    fn parse_utf8(&mut self) -> Result<CpInfo> {
        let length = self.read_u16()?;
        let mut bytes = vec![0u8; length as usize];
        self.r.read_exact(&mut bytes)?;

        Ok(CpInfo::Utf8(String::from_utf8_lossy(&bytes).into()))
    }

    fn parse_integer(&mut self) -> Result<CpInfo> {
        let int = self.read_i32()?;

        Ok(CpInfo::Integer(int))
    }

    // https://docs.oracle.com/javase/specs/jvms/se18/html/jvms-4.html#jvms-4.4.4
    fn parse_float(&mut self) -> Result<CpInfo> {
        let bits = self.read_u32()?;

        if bits == 0x7f800000 {
            // If bits is 0x7f800000, the float value will be positive infinity.
            todo!();
        } else if bits == 0xff800000 {
            // If bits is 0xff800000, the float value will be negative infinity.
            todo!();
        } else if (0x7f800001..=0x7fffffff).contains(&bits)
            || (0xff800001..=0xffffffff).contains(&bits)
        {
            // If bits is in the range 0x7f800001 through 0x7fffffff or in the range 0xff800001
            // through 0xffffffff, the float value will be NaN.
            todo!();
        }

        //  In all other cases, let s, e, and m be three values that might be computed from bits:
        let s: i32 = if (bits >> 31) == 0 { 1 } else { -1 };
        let e: i32 = (bits >> 23) as i32 & 0xff;
        let m: i32 = if e == 0 {
            ((bits & 0x7fffff) as i32) << 1
        } else {
            (bits & 0x7fffff) as i32 | 0x800000
        };

        Ok(CpInfo::Float(
            s as f32 * m as f32 * 2f32.powf(e as f32 - 150.),
        ))
    }

    fn parse_long(&mut self) -> Result<CpInfo> {
        let high_bytes = self.read_u32()?;
        let low_bytes = self.read_u32()?;

        Ok(CpInfo::Long(((high_bytes as i64) << 32) + low_bytes as i64))
    }

    fn parse_class_info(&mut self) -> Result<CpInfo> {
        let name_index = self.read_u16()?;

        Ok(CpInfo::Class(constant_pool::ClassInfo { name_index }))
    }

    fn parse_string(&mut self) -> Result<CpInfo> {
        let string_index = self.read_u16()?;

        Ok(CpInfo::String { string_index })
    }

    fn parse_field_ref(&mut self) -> Result<CpInfo> {
        let ref_info = self.parse_ref_info()?;

        Ok(CpInfo::FieldRef(ref_info))
    }

    fn parse_method_ref(&mut self) -> Result<CpInfo> {
        let ref_info = self.parse_ref_info()?;

        Ok(CpInfo::MethodRef(ref_info))
    }

    fn parse_interface_method_ref(&mut self) -> Result<CpInfo> {
        let ref_info = self.parse_ref_info()?;

        Ok(CpInfo::InterfaceMethodRef(ref_info))
    }

    fn parse_name_and_type_info(&mut self) -> Result<CpInfo> {
        let name_index = self.read_u16()?;
        let descriptor_index = self.read_u16()?;

        Ok(CpInfo::NameAndType(constant_pool::NameAndTypeInfo {
            name_index,
            descriptor_index,
        }))
    }

    fn parse_method_handle(&mut self) -> Result<CpInfo> {
        let reference_kind = self.read_u8()?;
        let reference_index = self.read_u16()?;

        Ok(CpInfo::MethodHandle(constant_pool::MethodHandleInfo {
            reference_kind,
            reference_index,
        }))
    }

    fn parse_method_type_info(&mut self) -> Result<CpInfo> {
        let descriptor_index = self.read_u16()?;

        Ok(CpInfo::MethodType(constant_pool::MethodTypeInfo {
            descriptor_index,
        }))
    }

    fn parse_invoke_dynamic_info(&mut self) -> Result<CpInfo> {
        let bootstrap_method_attr_index = self.read_u16()?;
        let name_and_type_index = self.read_u16()?;

        Ok(CpInfo::InvokeDynamic(constant_pool::InvokeDynamicInfo {
            bootstrap_method_attr_index,
            name_and_type_index,
        }))
    }

    fn parse_ref_info(&mut self) -> Result<constant_pool::RefInfo> {
        let class_index = self.read_u16()?;
        let name_and_type_index = self.read_u16()?;

        Ok(constant_pool::RefInfo {
            class_index,
            name_and_type_index,
        })
    }

    fn parse_attribute(&mut self) -> Result<Attribute> {
        let attribute_name_index = self.read_u16()?;
        let attribute_length = self.read_u32()?;
        let mut info = vec![0u8; attribute_length as usize];
        self.r.read_exact(&mut info)?;

        Ok(Attribute {
            attribute_name_index,
            info,
        })
    }

    pub fn parse_code_attribute(&mut self) -> Result<CodeAttribute> {
        let max_stack = self.read_u16()?;
        let max_locals = self.read_u16()?;
        let code_length = self.read_u32()?;
        let mut code = vec![0u8; code_length as usize];
        self.r.read_exact(&mut code)?;
        let exception_table_length = self.read_u16()?;
        let exception_table = (0..exception_table_length)
            .into_iter()
            .map(|_| self._parse_exception_table_entry())
            .collect::<Result<Vec<_>>>()?;
        let attributes_count = self.read_u16()?;
        let attributes = self.parse_attributes(attributes_count)?;

        Ok(CodeAttribute {
            max_stack,
            max_locals,
            code,
            exception_table,
            attributes,
        })
    }

    fn _parse_exception_table_entry(&mut self) -> Result<ExceptionTableEntry> {
        let start_pc = self.read_u16()?;
        let end_pc = self.read_u16()?;
        let handler_pc = self.read_u16()?;
        let catch_type = self.read_u16()?;

        Ok(ExceptionTableEntry {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        })
    }

    fn parse_attributes(&mut self, attributes_count: u16) -> Result<Attributes> {
        (0..attributes_count)
            .into_iter()
            .map(|_| self.parse_attribute())
            .collect::<Result<Vec<_>>>()
            .map(Attributes)
    }

    fn read_u32(&mut self) -> Result<u32> {
        Ok(self.r.read_u32::<Endian>()?)
    }

    fn read_u16(&mut self) -> Result<u16> {
        Ok(self.r.read_u16::<Endian>()?)
    }

    fn read_u8(&mut self) -> Result<u8> {
        Ok(self.r.read_u8()?)
    }

    fn read_i32(&mut self) -> Result<i32> {
        Ok(self.r.read_i32::<Endian>()?)
    }
}
