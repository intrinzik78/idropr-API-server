use crate::enums::Permission;

pub trait ToNumber {
    fn to_i8(self) -> i8;
    fn to_i16(self) -> i16;
    fn to_i32(self) -> i32;
    fn to_i64(self) -> i64;
    fn to_u8(self) -> u8;
    fn to_u16(self) -> u16;
    fn to_u32(self) -> u32;
    fn to_u64(self) -> u64;
}

impl ToNumber for Permission {
    fn to_i8(self) -> i8 {
        match self {
            Permission::Granted => 1,
            Permission::None => 0
        }
    }
    fn to_i16(self) -> i16 {
        match self {
            Permission::Granted => 1,
            Permission::None => 0
        }
    }
    fn to_i32(self) -> i32 {
        match self {
            Permission::Granted => 1,
            Permission::None => 0
        }
    }
    fn to_i64(self) -> i64 {
        match self {
            Permission::Granted => 1,
            Permission::None => 0
        }
    }
    fn to_u8(self) -> u8 {
        match self {
            Permission::Granted => 1,
            Permission::None => 0
        }
    }
    fn to_u16(self) -> u16 {
        match self {
            Permission::Granted => 1,
            Permission::None => 0
        }
    }
    fn to_u32(self) -> u32 {
        match self {
            Permission::Granted => 1,
            Permission::None => 0
        }
    }
    fn to_u64(self) -> u64 {
        match self {
            Permission::Granted => 1,
            Permission::None => 0
        }
    }
}