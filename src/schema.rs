// @generated automatically by Diesel CLI.

diesel::table! {
    aliases (id) {
        id -> Integer,
        address -> Varchar,
        goto -> Varchar,
        domain -> Varchar,
        created -> Datetime,
        modified -> Datetime,
        active -> Bool,
    }
}

diesel::table! {
    domains (id) {
        id -> Integer,
        domain -> Varchar,
        description -> Nullable<Varchar>,
        aliases -> Integer,
        mailboxes -> Integer,
        maxquota -> Bigint,
        quota -> Bigint,
        transport -> Nullable<Varchar>,
        backupmx -> Bool,
        created -> Datetime,
        modified -> Datetime,
        active -> Bool,
    }
}

diesel::table! {
    mailboxes (id) {
        id -> Integer,
        username -> Varchar,
        password -> Varchar,
        name -> Varchar,
        maildir -> Varchar,
        quota -> Bigint,
        domain -> Varchar,
        created -> Datetime,
        modified -> Datetime,
        active -> Bool,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Varchar,
        password -> Varchar,
        name -> Varchar,
        maildir -> Varchar,
        quota -> Bigint,
        domain -> Varchar,
        created -> Datetime,
        modified -> Datetime,
        active -> Bool,
    }
}

diesel::joinable!(aliases -> domains (domain));
diesel::joinable!(mailboxes -> domains (domain));
diesel::joinable!(users -> domains (domain));

diesel::allow_tables_to_appear_in_same_query!(
    aliases,
    domains,
    mailboxes,
    users,
); 
