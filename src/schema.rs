// @generated automatically by Diesel CLI.

diesel::table! {
    aliases (pkid) {
        pkid -> Integer,
        #[max_length = 255]
        mail -> Varchar,
        #[max_length = 255]
        destination -> Varchar,
        #[max_length = 255]
        domain -> Varchar,
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
        description -> Nullable<Varchar>,
        aliases -> Integer,
        maxquota -> Bigint,
        quota -> Bigint,
        #[max_length = 255]
        transport -> Nullable<Varchar>,
        backupmx -> Bool,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
    }
}

diesel::table! {
    users (pkid) {
        pkid -> Integer,
        #[max_length = 255]
        id -> Varchar,
        #[max_length = 255]
        password -> Varchar,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        maildir -> Varchar,
        quota -> Bigint,
        #[max_length = 255]
        domain -> Varchar,
        created -> Datetime,
        modified -> Datetime,
        enabled -> Bool,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    aliases,
    domains,
    users,
);
