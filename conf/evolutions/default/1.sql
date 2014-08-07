# Schema
 
# --- !Ups

create table aliases (
`pkid` smallint(3) NOT NULL auto_increment,
`mail` varchar(120) NOT NULL default '',
`destination` varchar(120) NOT NULL default '',
`enabled` boolean NOT NULL default '1',
PRIMARY KEY  (`pkid`),
UNIQUE KEY `mail` (`mail`)
) ;

CREATE TABLE domains (
`pkid` smallint(6) NOT NULL auto_increment,
`domain` varchar(120) NOT NULL default '',
`transport` varchar(120) NOT NULL default 'virtual:',
`enabled` boolean NOT NULL default '1',
PRIMARY KEY  (`pkid`)
) ;

CREATE TABLE users (
`id` varchar(128) NOT NULL default '',
`name` varchar(128) NOT NULL default '',
`uid` smallint(5) unsigned NOT NULL default '5000',
`gid` smallint(5) unsigned NOT NULL default '5000',
`home` varchar(255) NOT NULL default '/var/spool/mail/virtual',
`maildir` varchar(255) NOT NULL default 'blah/',
`enabled` boolean NOT NULL default '1',
`change_password` boolean NOT NULL default '1',
`clear` varchar(128) NOT NULL default 'ChangeMe',
`crypt` varchar(128) NOT NULL default 'sdtrusfX0Jj66',
`quota` varchar(255) NOT NULL default '',
PRIMARY KEY  (`id`),
UNIQUE KEY `id` (`id`)
) ;

CREATE TABLE backups (
	`pkid` smallint(6) NOT NULL auto_increment,
	`domain` varchar(128) NOT NULL default '',
	`transport` varchar(128) NOT NULL default ':[]',
	`enabled` boolean NOT NULL default '1',
	PRIMARY KEY  (`pkid`),
	UNIQUE KEY `domain` (`domain`)
);

CREATE TABLE relays (
	`pkid` smallint(6) NOT NULL auto_increment,
	`recipient` varchar(120) NOT NULL default '',
	`enabled` boolean NOT NULL default '1',
	`status` varchar(10) NOT NULL default 'OK',
	PRIMARY KEY  (`pkid`),
	UNIQUE KEY `recipient` (`recipient`)
);

CREATE TABLE relocated (
`pkid` smallint(6) NOT NULL auto_increment,
`oldadr` varchar(128) NOT NULL default '',
`newadr` varchar(128) NOT NULL default '',
`enabled` boolean NOT NULL default '1',
PRIMARY KEY  (`pkid`),
UNIQUE KEY `oldadr` (`oldadr`)
) ;

# --- !Downs

DROP TABLE IF EXISTS users;
DROP TABLE IF EXISTS aliases;
DROP TABLE IF EXISTS backups;
DROP TABLE IF EXISTS relays;
DROP TABLE IF EXISTS relocated;
DROP TABLE IF EXISTS domains;
