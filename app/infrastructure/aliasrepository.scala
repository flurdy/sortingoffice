package infrastructure

import javax.inject._
import org.joda.time.DateTime
import play.api.db.Database
import anorm._
import anorm.SqlParser._
import models._
import models.Environment.ConnectionName

@Singleton
class AliasRepository @Inject()(db: Database) {

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
      db.withConnection { implicit connection =>
         SQL(
            """
select * from aliases
where mail like {name}
order by mail
            """
         ).on(
            "name" -> s"%@${domain.name}"
         ).as(simpleAlias.*)
      }
   }

   def findAliases(aliases: List[String], domain: Domain): List[Alias] = {
      db.withConnection { implicit connection =>
         SQL(
            """
select * from aliases
where mail in ({name})
order by mail
            """
         ).on(
            "name" -> aliases.map( a => s"${a}@${domain.name}" )
         ).as(simpleAlias.*)
      }
   }


   def findAliases(connection: ConnectionName): List[Alias] = {
      db.withConnection { implicit connection =>
         SQL(
            """
select * from aliases
order by mail
            """
         ).as(simpleAlias.*)
      }
   }

   def findDomainAlias(alias: String, domain: Domain): Option[Alias] = {
      domain.connection.flatMap( connection => findAlias(connection,s"${alias}@${domain.name}") )
   }


   def findAlias(connection: ConnectionName, email: String): Option[Alias] = {
      db.withConnection { implicit connection =>
         SQL(
            """
select * from aliases
where mail = {name}
            """
         ).on(
            "name" -> email
         ).as(simpleAlias.*).headOption
      }
   }

   def findCatchAll(connectionName: ConnectionName, domain: Domain): Option[Alias] = findAlias(connectionName,s"@${domain.name}")

   def disable(connectionName: ConnectionName, email: String): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
               update aliases set enabled = 0 where mail = {email}
            """
         ).on(
            "email" -> email
         ).executeUpdate()
      }
   }

   def enable(connectionName: ConnectionName, email: String): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
               update aliases set enabled = 1 where mail = {email}
            """
         ).on(
            "email" -> email
         ).executeUpdate()
      }
   }

   def save(connectionName: ConnectionName, alias: Alias): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
insert into aliases (mail,destination,enabled) values ({mail},{destination},{enabled})
            """
         ).on(
            "mail" -> alias.mail,
            "destination" -> alias.destination,
            "enabled" -> alias.enabled
         ).execute()
      }
   }

   def delete(connectionName: ConnectionName, alias: Alias): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
delete from aliases where mail = {mail}
            """
         ).on(
            "mail" -> alias.mail
         ).execute()
      }
   }

   def updateDestination(connectionName: ConnectionName, alias: Alias): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
               update aliases set destination = {destination} where mail = {email}
            """
         ).on(
            "destination" -> alias.destination,
            "email" -> alias.mail
         ).executeUpdate()
      }
   }

}
