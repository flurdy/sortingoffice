// @generated automatically by Diesel CLI.

diesel::table! {
    aliases (pkid) {
        pkid -> Integer,
        #[max_length = 255]
        mail -> Varchar,
        #[max_length = 255]
        destination -> Varchar,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
    }
}

diesel::table! {
    backups (pkid) {
        pkid -> Integer,
        #[max_length = 255]
        domain -> Varchar,
        #[max_length = 255]
        transport -> Nullable<Varchar>,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
    }
}

diesel::table! {
    domains (pkid) {
        pkid -> Integer,
        #[max_length = 255]
        domain -> Varchar,
        #[max_length = 255]
        transport -> Nullable<Varchar>,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
    }
}

diesel::table! {
    relays (pkid) {
        pkid -> Integer,
        #[max_length = 255]
        recipient -> Varchar,
        #[max_length = 10]
        status -> Varchar,
        enabled -> Bool,
        created -> Datetime,
        modified -> Datetime,
    }
}

diesel::table! {
    users (pkid) {
        pkid -> Integer,
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 255]
        crypt -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        maildir -> Varchar,
        #[max_length = 255]
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
    domains,
    relays,
    users,
);
