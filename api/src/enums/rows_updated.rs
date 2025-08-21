// used by the ToUpdatedResult trait

#[derive(Clone,Copy,Debug)]
pub enum RowsUpdated {
    NoRowsUpdated,
    RowsUpdated(u64)
}