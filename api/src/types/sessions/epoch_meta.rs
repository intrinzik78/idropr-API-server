use sqlx::FromRow;

#[derive(Debug,FromRow)]
pub struct EpochMeta {
    pub user_id: i64,
    pub user_epoch: u64
}