Sorting Office
=========

A mail server user data management tool.

It assumes a mail server based on flurdy's "How to set up a mail server on a GNU / Linux system":
	http://flurdy.com/docs/postfix/


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


Soon
---
* External alias report
* Run examples
* Deltas across multiple databases reports


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
See separate RUN.md file


Hosting
----
See separate RUN.md file


Security
----
See separate SECURITY.md file

