use std::ops::Index;

#[derive(Debug, Default)]
pub struct ConstantPool {
    cp_infos: Vec<CpInfo>,
}
impl ConstantPool {
    pub fn new(cp_infos: Vec<CpInfo>) -> Self {
        Self { cp_infos }
    }
}
impl Index<u16> for ConstantPool {
    type Output = CpInfo;

    fn index(&self, index: u16) -> &Self::Output {
        &self.cp_infos[index as usize - 1]
    }
}
impl<'a> IntoIterator for &'a ConstantPool {
    type Item = &'a CpInfo;
    type IntoIter = std::slice::Iter<'a, CpInfo>;

    fn into_iter(self) -> Self::IntoIter {
        self.cp_infos.iter()
    }
}

#[macro_export]
macro_rules! matches_cp_info {
    ($cp:expr, $index:expr, $i:ident) => {
        match &$cp[$index] {
            crate::constant_pool::CpInfo::$i(ref n) => Ok(n),
            c => Err(crate::ClassFileError::UnexpectedConstantPoolEntry(
                stringify!($i),
                c.clone(),
            )),
        }
    };
}

#[derive(Debug, PartialEq, Clone)]
pub enum CpInfo {
    MethodRef(RefInfo),
    FieldRef(RefInfo),
    Float(f32),
    InterfaceMethodRef(RefInfo),
    Class(ClassInfo),
    NameAndType(NameAndTypeInfo),
    Utf8(String),
    String { string_index: u16 },
    InvokeDynamic(InvokeDynamicInfo),
    Integer(i32),
    MethodHandle(MethodHandleInfo),
    MethodType(MethodTypeInfo),
    Long(i64),
    Unusable,
}

#[derive(Debug, PartialEq, Clone)]
pub struct RefInfo {
    pub class_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassInfo {
    // The value of the name_index item must be a valid index into the constant_pool table.
    // The constant_pool entry at that index must be a CONSTANT_Utf8_info structure (§4.4.7)
    // representing a valid binary class or interface name encoded in internal form (§4.2.1).
    pub name_index: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NameAndTypeInfo {
    pub name_index: u16,
    pub descriptor_index: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InvokeDynamicInfo {
    pub bootstrap_method_attr_index: u16,
    pub name_and_type_index: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodHandleInfo {
    pub reference_kind: u8,
    pub reference_index: u16,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodTypeInfo {
    pub descriptor_index: u16,
}
