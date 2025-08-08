use serde::Serialize;

use crate::types::ApiResponse;

#[derive(Serialize, Debug)]
pub enum ApiResult<T>
where T: Serialize
{
    Ok(ApiResponse<T>),
    Error(ApiResponse<T>),
}