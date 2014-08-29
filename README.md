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


Soon
---
* Edit domains, aliases and users.


Maybe
---
* Caching for frequent data.
* TLS encryption
* Password resets
* Dynamically add server
* LDAP authentication
* Check DNS entries for relevant domain(s).
* Application authentication roles


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

Unmodified the application will run the internal memory database as an example.

Modify your own version of application.conf with your own:

* Databases
* Database features
* Common cross aliases
* Application user(s)

And then run the application with

	activator -Dconfig.file=conf/yoursettings.conf

Remember at the top of your file to always include the line:

	include "application-default"


Hosting
----

It is out of scope to describe in detail how you host this application.

This application is a Play! 2.x Scala based application and requires access to your mail databases.

* https://www.playframework.com/documentation/2.2.x/Production
* http://flurdy.com/docs/scalainit/startscala.html

Please also read the security tips below.


Security: encryption, authentication and authorisation
------

This is admin application and should never be exposed directly on the internet.

Two resources can and should be secured:

* Access to the application
* Access to the databases

Application authentication
---

Using built in authentication. Users and credentials specified in a properties file.

Application authorisation
----

The application does not have any plans for application level authorisation feature.


Application encryption
----

Apache or nginx can be put infront of the application to provide SSL/TLS transport encryption.

Database authentication
---

Normal MySQL authentication. Passwords are stored in configuration files. These can be an environment property.


Database authorisation
---

The database user that the application connects with can be given different database access levels.


Database encryption
---

The passwords are encrypted using the howtos/postfix's encryption, and are not used in this application.

You can connect the database via unencrypted standard mysql port, via MySQL's SSL connection feature or via a SSH tunnel.

If you deploy to a PAAS such as Heroku, you need to add SSL authentication to your AWS or elsewhere hosted database. You can modify Heroku's RDS suggestions to your MySQL solution: https://devcenter.heroku.com/articles/amazon_rds#require-ssl

Security suggestions
----

* Don't run the application on the same server as the database.
* Don't expose the MySQL port unencrypted.
* Don't expose this application to everyone.
* You can configure the application with a read only database user.


