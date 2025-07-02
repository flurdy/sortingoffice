// @generated automatically by Diesel CLI.

diesel::table! {
    aliases (id) {
        id -> Int,
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
        id -> Int,
        domain -> Varchar,
        description -> Nullable<Varchar>,
        aliases -> Int,
        mailboxes -> Int,
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
        id -> Int,
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
        id -> Int,
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
