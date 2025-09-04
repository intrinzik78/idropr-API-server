// used by the ToAffectedResult trait

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum RowsAffected {
    None,
    Some(u64)
}