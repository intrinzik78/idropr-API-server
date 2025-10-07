use serde::Serialize;
use utoipa::ToSchema;

use crate::types::ApiResponse;

#[derive(Serialize,Debug,ToSchema)]
pub enum ApiResult<T>
where T: Serialize
{
    Ok(ApiResponse<T>),
    Error(ApiResponse<T>),
}