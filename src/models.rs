use diesel::prelude::*;
use serde::{Deserialize, Serialize, Deserializer};
use chrono::NaiveDateTime;
use crate::schema::*;

fn deserialize_checkbox<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(matches!(opt.as_deref(), Some("on") | Some("true") | Some("1")))
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = domains)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Domain {
    pub pkid: i32,
    pub domain: String,
    pub description: Option<String>,
    pub aliases: i32,
    pub maxquota: i64,
    pub quota: i64,
    pub transport: Option<String>,
    pub backupmx: bool,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = domains)]
pub struct NewDomain {
    pub domain: String,
    pub description: Option<String>,
    pub aliases: i32,
    pub maxquota: i64,
    pub quota: i64,
    pub transport: Option<String>,
    pub backupmx: bool,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, Identifiable)]
#[diesel(table_name = users)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct User {
    pub pkid: i32,
    pub username: String,
    pub password: String,
    pub name: String,
    pub maildir: String,
    pub quota: i64,
    pub domain: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = users)]
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
#[diesel(table_name = aliases)]
#[diesel(primary_key(pkid))]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
pub struct Alias {
    pub pkid: i32,
    pub mail: String,
    pub destination: String,
    pub domain: String,
    pub created: NaiveDateTime,
    pub modified: NaiveDateTime,
    pub active: bool,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = aliases)]
pub struct NewAlias {
    pub mail: String,
    pub destination: String,
    pub domain: String,
    pub active: bool,
}



// Form data structures for HTMX
#[derive(Deserialize, Debug)]
pub struct DomainForm {
    pub domain: String,
    #[serde(default)]
    pub description: String,
    pub aliases: i32,
    pub maxquota: i64,
    pub quota: i64,
    pub transport: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub backupmx: bool,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub active: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct UserForm {
    pub username: String,
    pub password: String,
    pub name: String,
    pub domain: String,
    pub quota: i64,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub active: bool,
}

#[derive(Debug, Deserialize)]
pub struct AliasForm {
    pub mail: String,
    pub destination: String,
    pub domain: String,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_checkbox")]
    pub active: bool,
}



// Statistics models
#[derive(Debug, Serialize, Deserialize)]
pub struct DomainStats {
    pub domain: String,
    pub user_count: i64,
    pub alias_count: i64,
    pub total_quota: i64,
    pub used_quota: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemStats {
    pub total_domains: i64,
    pub total_users: i64,
    pub total_aliases: i64,
    pub total_quota: i64,
    pub used_quota: i64,
} 
