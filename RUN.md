
Run
=====


Local
-----

Unmodified the application will run the internal memory database as an example.

So just run locally with:

   activator

Create and modify your own version of application.conf to customise.
Remember at the top of your file to always include the line:

   include "application-default"

Configurable features:

* Databases
* Database features
* Common cross aliases
* Application user(s)

For testing and real data development you can export some production data,
import them into a local MySQL database server
and test the application against that database(s).

   activator -Dconfig.file=conf/test.conf

If you do not want a permanent hosting of the application for the management of production mail servers,
you could create SSH tunnels between your machine and the production MySQL server instead.

   ssh -gvNL 3306:localhost:3306 productionmailhost.example.com

   activator -Dconfig.file=conf/prod.conf

If you tunnel is not the actual machine you application runs on you may need to allow external ports on your local machine by adding to /etc/ssh/sshd_config

   GatewayPorts yes


Docker
-----

The application includes a Dockerfile file. And a Docker image is built on every push and is available at https://registry.hub.docker.com/u/flurdy/sortingoffice/

To run Docker with sortingoffice:

   docker -ti --rm -p 49900:9000 flurdy/sortingoffice run

To run with your own configuration:

   docker -ti --rm -p 49900:9000 \
      -v /your/local/path/folder:/etc/opt/sortingoffice \
      flurdy/sortingoffice \
      -Dconfig.file=/etc/opt/sortingoffice/mydocker.conf \
      run

You could run a MySQL server as a Docker container and import the data into it.
( https://registry.hub.docker.com/u/flurdy/mysql/ )
To run with an already containerised database :

   docker -ti --rm -p 49900:9000 \
      -v /your/local/path/folder:/etc/opt/sortingoffice \
      --link maildb:maildb \
      flurdy/sortingoffice \
      -Dconfig.file=/etc/opt/sortingoffice/mydocker.conf \
      run



Hosting
=====

It is out of scope to describe in detail how you host this application.

This application is a Play! 2.x Scala based application and requires access to your mail databases.

* https://www.playframework.com/documentation/2.2.x/Production
* http://flurdy.com/docs/scalainit/startscala.html

Please also read the security advice in SECURITY.md.





