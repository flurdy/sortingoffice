# Bootstrap data

# --- !Ups

insert into domains (domain,enabled) values ('example.com',1);
insert into domains (domain,enabled) values ('example.net',1);
insert into domains (domain,enabled) values ('example.org',0);

insert into backups (domain,transport,enabled) values ('example.co.uk',':[smtp.example.com]',1);
insert into backups (domain,transport,enabled) values ('example.org',':[smtp.example.com]',0);

insert into relays (recipient) values ('@example.co.uk');
insert into relays (recipient) values ('@example.net');
insert into relays (recipient,enabled) values ('@example.com',0);
insert into relays (recipient,status) values ('spam@example.com','REJECT');

insert into aliases (mail,destination) values ('@example.net','@example.com');
insert into aliases (mail,destination,enabled) values ('you@example.net','me@example.com',0);
insert into aliases (mail,destination) values ('postmaster@example.com','me@example.com');
insert into aliases (mail,destination) values ('abuse@example.com','me@example.com');
insert into aliases (mail,destination) values ('me@example.com','me@example.com');

insert into users (id,name,maildir,enabled,change_password) values ('me@example.com','me','example.com/me',1,0);
insert into users (id,name,maildir,enabled) values ('you@example.com','you','example.com/you',0);

# --- !Downs

delete from domains;
delete from backups;
delete from relocated;
delete from relays;
delete from aliases;
delete from users;
