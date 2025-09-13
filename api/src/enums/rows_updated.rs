// used by the ToUpdatedResult trait

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RowsUpdated {
    None,
    Some(u64)
}