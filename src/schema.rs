// @generated automatically by Diesel CLI.

diesel::table! {
    aliases (id) {
        id -> Integer,
        address -> Varchar,
        goto -> Varchar,
        domain -> Varchar,
        active -> Bool,
        created -> Datetime,
        modified -> Datetime,
    }
}

diesel::table! {
    domains (id) {
        id -> Integer,
        domain -> Varchar,
        description -> Varchar,
        aliases -> Integer,
        mailboxes -> Integer,
        maxquota -> Bigint,
        quota -> Bigint,
        transport -> Varchar,
        backupmx -> Bool,
        active -> Bool,
        created -> Datetime,
        modified -> Datetime,
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
        active -> Bool,
        created -> Datetime,
        modified -> Datetime,
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
        active -> Bool,
        created -> Datetime,
        modified -> Datetime,
    }
}

// diesel::joinable!(aliases -> domains (domain));
// diesel::joinable!(mailboxes -> domains (domain));
// diesel::joinable!(users -> domains (domain));

diesel::allow_tables_to_appear_in_same_query!(
    aliases,
    domains,
    mailboxes,
    users,
); 
