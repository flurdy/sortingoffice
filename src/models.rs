use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::domains)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Domain {
    pub id: i32,
    pub domain: String,
    pub description: Option<String>,
    pub aliases: i32,
    pub mailboxes: i32,
    pub maxquota: i64,
    pub quota: i64,
    pub transport: Option<String>,
    pub backupmx: bool,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::domains)]
pub struct NewDomain {
    pub domain: String,
    pub description: Option<String>,
    pub aliases: i32,
    pub mailboxes: i32,
    pub maxquota: i64,
    pub quota: i64,
    pub transport: Option<String>,
    pub backupmx: bool,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub name: String,
    pub maildir: String,
    pub quota: i64,
    pub domain: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::users)]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub name: String,
    pub maildir: String,
    pub quota: i64,
    pub domain: String,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::aliases)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Alias {
    pub id: i32,
    pub address: String,
    pub goto: String,
    pub domain: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::aliases)]
pub struct NewAlias {
    pub address: String,
    pub goto: String,
    pub domain: String,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::mailboxes)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Mailbox {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub name: String,
    pub maildir: String,
    pub quota: i64,
    pub domain: String,
    pub created: DateTime<Utc>,
    pub modified: DateTime<Utc>,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::mailboxes)]
pub struct NewMailbox {
    pub username: String,
    pub password: String,
    pub name: String,
    pub maildir: String,
    pub quota: i64,
    pub domain: String,
    pub active: bool,
}

// Form data structures for HTMX
#[derive(Debug, Deserialize)]
pub struct DomainForm {
    pub domain: String,
    pub description: Option<String>,
    pub aliases: i32,
    pub mailboxes: i32,
    pub maxquota: i64,
    pub quota: i64,
    pub transport: Option<String>,
    pub backupmx: bool,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct UserForm {
    pub username: String,
    pub password: String,
    pub name: String,
    pub domain: String,
    pub quota: i64,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct AliasForm {
    pub address: String,
    pub goto: String,
    pub domain: String,
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct MailboxForm {
    pub username: String,
    pub password: String,
    pub name: String,
    pub domain: String,
    pub quota: i64,
    pub active: bool,
}

// Statistics models
#[derive(Debug, Serialize)]
pub struct DomainStats {
    pub domain: String,
    pub user_count: i64,
    pub alias_count: i64,
    pub mailbox_count: i64,
    pub total_quota: i64,
    pub used_quota: i64,
}

#[derive(Debug, Serialize)]
pub struct SystemStats {
    pub total_domains: i64,
    pub total_users: i64,
    pub total_aliases: i64,
    pub total_mailboxes: i64,
    pub total_quota: i64,
    pub used_quota: i64,
} 
