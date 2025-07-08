// @generated automatically by Diesel CLI.

diesel::table! {
    aliases (pkid) {
        pkid -> Integer,
        mail -> Varchar,
        destination -> Varchar,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
    }
}

diesel::table! {
    backups (pkid) {
        pkid -> Integer,
        domain -> Varchar,
        transport -> Nullable<Varchar>,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
    }
}

diesel::table! {
    clients (id) {
        id -> Integer,
        client -> Varchar,
        status -> Varchar,
        created_at -> Nullable<Timestamp>,
        updated_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    domains (pkid) {
        pkid -> Integer,
        domain -> Varchar,
        transport -> Nullable<Varchar>,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
    }
}

diesel::table! {
    relays (pkid) {
        pkid -> Integer,
        recipient -> Varchar,
        status -> Varchar,
        enabled -> Bool,
        created -> Datetime,
        modified -> Datetime,
    }
}

diesel::table! {
    relocated (pkid) {
        pkid -> Integer,
        old_address -> Varchar,
        new_address -> Varchar,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
    }
}

diesel::table! {
    users (pkid) {
        pkid -> Integer,
        id -> Varchar,
        crypt -> Varchar,
        name -> Varchar,
        maildir -> Varchar,
        home -> Varchar,
        uid -> Unsigned<Smallint>,
        gid -> Unsigned<Smallint>,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
        change_password -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    aliases,
    backups,
    clients,
    domains,
    relays,
    relocated,
    users,
);
