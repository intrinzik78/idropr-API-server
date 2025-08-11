// external libraries
use sqlx::{FromRow, MySql, Transaction};

// internal libraries
use crate::{
    enums::{Error,Permission},
    traits::{ToNumber,ToPermission},
    types::DatabaseConnection
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoftwareAccess {
    pub accounts: Permission,
    pub api: Permission,
    pub users: Permission,
    pub bookings: Permission,
    pub clients: Permission,
    pub leads: Permission,
    pub surveys: Permission,
    pub reports: Permission,
    pub sales: Permission,
}

#[derive(Debug, Clone, FromRow)]
struct DatabaseHelper {
    api: i8,
    accounts: i8,
    users: i8,
    bookings: i8,
    clients: i8,
    leads: i8,
    surveys: i8,
    reports: i8,
    sales: i8,
}

impl DatabaseHelper {
    pub fn transform(self) -> SoftwareAccess {
        SoftwareAccess {
            api: self.api.to_permission(),
            accounts: self.accounts.to_permission(),
            users: self.users.to_permission(),
            bookings: self.bookings.to_permission(),
            clients: self.clients.to_permission(),
            leads: self.leads.to_permission(),
            surveys: self.surveys.to_permission(),
            reports: self.reports.to_permission(),
            sales: self.sales.to_permission(),
        }
    }
}

// async
impl SoftwareAccess {
    pub async fn into_db_as_transaction(user_id: i64, access_rights: SoftwareAccess, tx: &mut Transaction<'static,MySql>) -> Result<u64> {
        let accounts = access_rights.accounts.to_i8();
        let api = access_rights.api.to_i8();
        let bookings = access_rights.bookings.to_i8();
        let clients = access_rights.clients.to_i8();
        let leads = access_rights.leads.to_i8();
        let users = access_rights.users.to_i8();
        let surveys = access_rights.surveys.to_i8();
        let reports = access_rights.reports.to_i8();
        let sales = access_rights.sales.to_i8();

        let sql = "INSERT INTO `software_access` (id,accounts,api,users,bookings,clients,leads,surveys,reports,sales) VALUES(?,?,?,?,?,?,?,?,?,?)";
        let insert_id = sqlx::query(sql)
            .bind(user_id)
            .bind(accounts)
            .bind(api)
            .bind(users)
            .bind(bookings)
            .bind(clients)
            .bind(leads)
            .bind(surveys)
            .bind(reports)
            .bind(sales)
            .execute(&mut **tx)
            .await?
            .last_insert_id();

        Ok(insert_id)
    }

    pub async fn from_user_id(user_id: i64,database: &DatabaseConnection) -> Result<SoftwareAccess> {
        let sql = "SELECT accounts,api,users,bookings,clients,leads,surveys,reports,sales FROM `software_access` WHERE id = ? LIMIT 1";
        let helper: DatabaseHelper = sqlx::query_as(sql)
            .bind(user_id)
            .fetch_one(&database.pool)
            .await?;

        let software_access = helper.transform();

        Ok(software_access)
    }
}

// builder functions
impl SoftwareAccess {
    pub fn with_accounts(mut self) -> Self {
        self.accounts = Permission::Granted;
        self
    }

    pub fn with_api(mut self) -> Self {
        self.api = Permission::Granted;
        self
    }

    pub fn with_users(mut self) -> Self {
        self.users = Permission::Granted;
        self
    }

    pub fn with_bookings(mut self) -> Self {
        self.bookings = Permission::Granted;
        self
    }

    pub fn with_clients(mut self) -> Self {
        self.clients = Permission::Granted;
        self
    }

    pub fn with_leads(mut self) -> Self {
        self.leads = Permission::Granted;
        self
    }

    pub fn with_surveys(mut self) -> Self {
        self.surveys = Permission::Granted;
        self
    }

    pub fn with_reports(mut self) -> Self {
        self.reports = Permission::Granted;
        self
    }

    pub fn with_sales(mut self) -> Self {
        self.sales = Permission::Granted;
        self
    }
}

impl Default for SoftwareAccess {
    fn default() -> Self {
        SoftwareAccess {
            accounts: Permission::None,
            users: Permission::None,
            bookings: Permission::None,
            clients: Permission::None,
            leads: Permission::None,
            surveys: Permission::None,
            reports: Permission::None,
            sales: Permission::None,
            api: Permission::None
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    // tests the default build has all required permissions and they are set to Permission::None
    #[test]
    fn default_permission_builder() {
        let rights = SoftwareAccess {
            accounts: Permission::None,
            api: Permission::None,
            users: Permission::None,
            bookings: Permission::None,
            clients: Permission::None,
            leads: Permission::None,
            surveys: Permission::None,
            reports: Permission::None,
            sales: Permission::None,
        };
        let build_test = SoftwareAccess::default();
        assert_eq!(rights,build_test);
    }

    // tests the builder can set all required permissions to Permission::Granted
    #[test]
    fn default_with_full_permissions() {
        let rights = SoftwareAccess {
            accounts: Permission::Granted,
            api: Permission::Granted,
            users: Permission::Granted,
            bookings: Permission::Granted,
            clients: Permission::Granted,
            leads: Permission::Granted,
            surveys: Permission::Granted,
            reports: Permission::Granted,
            sales: Permission::Granted,
        };
        let build_test = SoftwareAccess::default()
            .with_accounts()
            .with_api()
            .with_bookings()
            .with_clients()
            .with_leads()
            .with_reports()
            .with_sales()
            .with_surveys()
            .with_users();

        assert_eq!(rights,build_test);
    }
}