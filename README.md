Sorting Office
=========

A mail server user data management tool.

It assumes a mail server based on flurdy's "How to set up a mail server on a GNU / Linux system":
	http://flurdy.com/docs/postfix/


Versions
----

### v1.x

Version 1 is the stable and most feature rich version.

It is currently on master, ie. this branch.

It is the one you should use.


### v2.x

Version 2 is in development, unstable, and currently with very limited features.

It is on branch v2.x until more stable.

It is a rethink of how to approach multiple but related mail servers.



Features
----
* View data that are contained in the mail database.
* View data on several mail databases.
* View which domains a server are responsible for.
* View which domains are backed up if enabled.
* View aliases to forward emails.
* View alias relays to accept if enabled.
* View users that will store emails on a server.
* Toggle between enable and disable for domains, aliases and users.
* Add new domains, backup domains, aliases, relays and users.
* Remove domains, backup domains, aliases, relays and users.
* Authentication for application access.
* Edit domains, aliases, relays and users.
* Check domains, aliases and relays across multiple databases
* Docker image available


Soon
---
* External alias report


Maybe
---
* Caching for frequent data.
* TLS encryption
* Password resets
* Dynamically add server
* LDAP authentication
* Check DNS entries for relevant domain(s).
* Application authentication roles
* Transfer domains between databases


Not
---

* Not read or modify any files on the server.
* This will not configure your server's configurations.
* Nor read, monitor or alter any actual emails.


Source code
-----
* http://github.com/flurdy/sortingoffice


Live demo
-----
* http://sortingoffice-demo.herokuapp.com

Application users are john and mary with 123 as password


Run
-----
See separate [RUN.md](RUN.md) file


Hosting
----
See separate [RUN.md](RUN.md) file


Security
----
See separate [SECURITY.md](SECURITY.md) file
