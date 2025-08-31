// used by the ToUpdatedResult trait

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RowsUpdated {
    NoRowsUpdated,
    RowsUpdated(u64)
}