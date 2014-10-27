FROM flurdy/activator-mini:latest

MAINTAINER flurdy

ENV DEBIAN_FRONTEND noninteractive

ENV GITHUBUSER flurdy
ENV APPBRANCH master
ENV APPNAME sortingoffice
ENV APPDIR /var/local/sortingoffice

RUN wget https://github.com/$GITHUBUSER/$APPNAME/archive/$APPBRANCH.zip && \
  unzip $APPBRANCH.zip && \
  rm $APPBRANCH.zip && \
  mv $APPNAME-$APPBRANCH $APPDIR

WORKDIR /var/local/sortingoffice

RUN activator clean compile dist && \
  rm $APPDIR/target/universal/$APPNAME-*.zip

EXPOSE 9000
EXPOSE 9999

ENTRYPOINT ["/usr/local/bin/activator"]

CMD ["run"]

## To run image :
# docker run -ti --rm -p 49900:9000 \
#   flurdy/sortingoffice \
#   run
#
## Complex alternative :
# docker run -ti --rm -p 49910:9000 \
#   -v /path/to/myconffolder:/etc/opt/sortingoffice:r \
#   --link maildb:maildb \
#   flurdy/sortingoffice \
#   -Dconfig.file=/etc/opt/sortingoffice/my.conf \
#   run
#
