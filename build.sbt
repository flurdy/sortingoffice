enablePlugins(PlayScala)

name := "sortingoffice"

version := "1.0-SNAPSHOT"

scalaVersion := "3.3.1"

libraryDependencies ++= Seq(
  "org.playframework" %% "play" % "3.0.0",
  "org.playframework.anorm" %% "anorm" % "2.7.0",
  "org.playframework" %% "play-jdbc" % "3.0.0",
  "org.playframework" %% "play-cache" % "3.0.0",
  "org.webjars" % "jquery" % "3.6.4",
  "org.webjars" % "bootstrap" % "5.3.2",
  "mysql" % "mysql-connector-java" % "8.0.33",
  "com.github.t3hnar" % "scala-bcrypt_2.13" % "4.3.0",
  "org.apache.pekko" %% "pekko-stream" % "1.0.2"
)
