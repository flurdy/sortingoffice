Security: encryption, authentication and authorisation
=====

This is admin application and should never be exposed directly on the internet.

Two resources can and should be secured:

* Access to the application
* Access to the databases

#### Application authentication


Using built in authentication. Users and credentials specified in a properties file.

#### Application authorisation


The application does not have any plans for application level authorisation feature.


#### Application encryption


Apache or nginx can be put infront of the application to provide SSL/TLS transport encryption.

#### Database authentication


Normal MySQL authentication. Passwords are stored in configuration files. These can be environment properties.


#### Database authorisation


The database user that the application connects with can be given different database access levels.


#### Database encryption


The mail user's passwords are encrypted using the howtos/postfix's encryption, and are not used at all in this application.

You can connect the database via unencrypted standard mysql port, via MySQL's SSL connection feature or via a SSH tunnel.

If you deploy to a PAAS such as Heroku, you need to add SSL authentication to your AWS or elsewhere hosted database. You can modify Heroku's RDS suggestions to your MySQL solution: https://devcenter.heroku.com/articles/amazon_rds#require-ssl

Security suggestions
----

* Don't run the application on the same server as the database.
* Don't expose the MySQL port unencrypted.
* Don't expose this application to everyone.
* You can configure the application with a read only database user.


