// a fast and efficient way to determine whether a user has the appropriate permissions set to access an API endpoint
use crate::{
    enums::Permission,
    types::SoftwareAccess
};

// creates bit flags for each permission toggle
const ACCOUNTS:u16 =  0b0000_0000_0001;
const API:u16 =       0b0000_0000_0010;
const USERS:u16 =     0b0000_0000_0100;
const BOOKINGS:u16 =  0b0000_0000_1000;
const CLIENTS:u16 =   0b0000_0001_0000;
const LEADS:u16 =     0b0000_0010_0000;
const SURVEYS:u16 =   0b0000_0100_0000;
const REPORTS:u16 =   0b0000_1000_0000;
const SALES:u16 =     0b0001_0000_0000;

// single method called on a user's SoftwareAccess struct. 
// Required rights are passed into the method where a comparison is made and Pass/Fail result returned.
pub trait HasPermission {
    fn has_permission(self, required_rights: &SoftwareAccess) -> Permission;
}

impl HasPermission for &SoftwareAccess {
    fn has_permission(self, required_rights: &SoftwareAccess) -> Permission {
        let mut user_flags = 0_u16;

        if self.accounts == Permission::Granted { user_flags |= ACCOUNTS; }
        if self.api == Permission::Granted      { user_flags |= API; }
        if self.users == Permission::Granted    { user_flags |= USERS; }
        if self.bookings == Permission::Granted { user_flags |= BOOKINGS; }
        if self.clients == Permission::Granted  { user_flags |= CLIENTS; }
        if self.leads == Permission::Granted    { user_flags |= LEADS; }
        if self.surveys == Permission::Granted  { user_flags |= SURVEYS; }
        if self.reports == Permission::Granted  { user_flags |= REPORTS; }
        if self.sales == Permission::Granted    { user_flags |= SALES; }

        let mut required_flags = 0_u16;

        if required_rights.accounts == Permission::Granted { required_flags |= ACCOUNTS; }
        if required_rights.api == Permission::Granted      { required_flags |= API; }
        if required_rights.users == Permission::Granted    { required_flags |= USERS; }
        if required_rights.bookings == Permission::Granted { required_flags |= BOOKINGS; }
        if required_rights.clients == Permission::Granted  { required_flags |= CLIENTS; }
        if required_rights.leads == Permission::Granted    { required_flags |= LEADS; }
        if required_rights.surveys == Permission::Granted  { required_flags |= SURVEYS; }
        if required_rights.reports == Permission::Granted  { required_flags |= REPORTS; }
        if required_rights.sales == Permission::Granted    { required_flags |= SALES; }

        // bitwise comparison
        if required_flags & user_flags == required_flags {
            Permission::Granted
        } else {
            Permission::None
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    // tests bit-flag check for granting permissions
    fn account_permissions() {
        // creates test users
        let user_with_no_rights = SoftwareAccess::default();
        let user_with_all_rights = SoftwareAccess::default()
            .with_accounts()
            .with_api()
            .with_bookings()
            .with_clients()
            .with_leads()
            .with_reports()
            .with_sales()
            .with_surveys()
            .with_users();

        // creates permission requirements to test users against
        let user_accounts = SoftwareAccess::default().with_accounts();
        let user_api = SoftwareAccess::default().with_api();
        let user_users = SoftwareAccess::default().with_users();
        let user_bookings = SoftwareAccess::default().with_bookings();
        let user_clients = SoftwareAccess::default().with_clients();
        let user_leads = SoftwareAccess::default().with_leads();
        let user_surveys = SoftwareAccess::default().with_surveys();
        let user_reports = SoftwareAccess::default().with_reports();
        let user_sales = SoftwareAccess::default().with_sales();

        //fail case
        assert_eq!(user_with_no_rights.has_permission(&user_accounts), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&user_api), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&user_users), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&user_bookings), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&user_clients), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&user_leads), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&user_surveys), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&user_reports), Permission::None);
        assert_eq!(user_with_no_rights.has_permission(&user_sales), Permission::None);

        //success case
        assert_eq!(user_with_all_rights.has_permission(&user_accounts), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&user_api), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&user_users), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&user_bookings), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&user_clients), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&user_leads), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&user_surveys), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&user_reports), Permission::Granted);
        assert_eq!(user_with_all_rights.has_permission(&user_sales), Permission::Granted);
    
    }
}