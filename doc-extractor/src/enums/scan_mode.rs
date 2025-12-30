use utoipa::ToSchema;

#[derive(Debug,ToSchema)]
#[repr(u8)]
pub enum ScanMode {
    Low     = 1,
    High    = 2
}