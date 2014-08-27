name := "sortingoffice"

version := "1.0-SNAPSHOT"

libraryDependencies ++= Seq(
  jdbc,
  anorm,
  cache,
  "org.webjars" %% "webjars-play" % "2.2.1",
  "org.webjars" % "jquery" % "2.0.3-1",
  "org.webjars" % "bootstrap" % "3.0.3",
  "mysql" % "mysql-connector-java" % "5.1.27",
  "com.github.t3hnar" %% "scala-bcrypt" % "2.4"
)

play.Project.playScalaSettings
