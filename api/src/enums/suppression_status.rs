use serde::Deserialize;

use crate::enums::Error;

#[repr(u8)]
#[derive(Clone,Debug,Deserialize,PartialEq)]
pub enum SuppressionStatus {
    Hard = 0,
    Soft = 1,
    SpamComplaint = 2,
    ManualSuppression = 3,
    PolicyBlock = 4,
    Unsubscribe = 5
}

impl SuppressionStatus {
    pub fn from_u8(num:u8) -> Result<Self,Error> {
        Ok(match num {
            0 => Self::Hard,
            1 => Self::Soft,
            2 => Self::SpamComplaint,
            3 => Self::ManualSuppression,
            4 => Self::PolicyBlock,
            5 => Self::Unsubscribe,
            _ => return Err(Error::SuppressionStatusOutOfRange)
        })
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn bounds_check() {
        type S = SuppressionStatus;

        let hard = S::Hard;
        let soft = S::Soft;
        let spam_complaint = S::SpamComplaint;
        let manual_suppression = S::ManualSuppression;
        let policy_block = S::PolicyBlock;
        let unsubscribe = S::Unsubscribe;

        let hard_test = S::from_u8(0).unwrap();
        let soft_test = S::from_u8(1).unwrap();
        let spam_complaint_test = S::from_u8(2).unwrap();
        let manual_suppression_test = S::from_u8(3).unwrap();
        let policy_block_test = S::from_u8(4).unwrap();
        let unsubscribe_test = S::from_u8(5).unwrap();
        let fail_test = S::from_u8(6);

        assert_eq!(hard,hard_test);
        assert_eq!(soft,soft_test);
        assert_eq!(spam_complaint,spam_complaint_test);
        assert_eq!(manual_suppression,manual_suppression_test);
        assert_eq!(policy_block,policy_block_test);
        assert_eq!(unsubscribe,unsubscribe_test);

        assert!(fail_test.is_err());
    }
}
