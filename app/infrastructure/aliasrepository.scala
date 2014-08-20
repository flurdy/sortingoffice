package infrastructure

import play.api.Play.current
import play.api.Logger
import org.joda.time.DateTime
import play.api.db.DB
import anorm._
import anorm.SqlParser._
import models._
import models.Environment.ConnectionName


object AliasRepository {

   val simpleAlias = {
      get[String]("mail") ~
      get[String]("destination") ~
      get[Boolean]("enabled") map {
         case mail~destination~enabled => {
            Alias(mail, destination, enabled)
         }
      }
   }

   def findAllAliasesForDomain(domain: Domain): List[Alias] = {
      DB.withConnection(domain.connection.get) { implicit connection =>
         SQL(
            """
select * from aliases
where mail like {name}
order by mail
            """
         ).on(
            'name -> s"%@${domain.name}"
         ).as(simpleAlias *)
      }
   }

   def findAliases(aliases: List[String], domain: Domain): List[Alias] = {
      DB.withConnection(domain.connection.get) { implicit connection =>
         SQL(
            """
select * from aliases
where mail in ({name})
order by mail
            """
         ).on(
            'name -> aliases.map( a => s"${a}@${domain.name}" )
         ).as(simpleAlias *)
      }
   }

   def findDomainAlias(alias: String, domain: Domain): Option[Alias] = {
      domain.connection flatMap( connection => findAlias(connection,s"${alias}@${domain.name}") )
   }


   def findAlias(connection: ConnectionName, email: String): Option[Alias] = {
      DB.withConnection(connection) { implicit connection =>
         SQL(
            """
select * from aliases
where mail = {name}
            """
         ).on(
            'name -> email
         ).as(simpleAlias *).headOption
      }
   }

   def findCatchAll(connectionName: ConnectionName, domain: Domain): Option[Alias] = findAlias(connectionName,s"@${domain.name}")

   def disable(connectionName: ConnectionName, email: String) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
               update aliases set enabled = 0 where mail = {email}
            """
         ).on(
            'email -> email
         ).executeUpdate
      }
   }

   def enable(connectionName: ConnectionName, email: String) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
               update aliases set enabled = 1 where mail = {email}
            """
         ).on(
            'email -> email
         ).executeUpdate
      }
   }

}
