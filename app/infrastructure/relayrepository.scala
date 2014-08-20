package infrastructure

import play.api.Play.current
import play.api.Logger
import org.joda.time.DateTime
import play.api.db.DB
import anorm._
import anorm.SqlParser._
import models._
import models.Environment.ConnectionName


object RelayRepository {

   val simpleRelay = {
      get[String]("recipient") ~
      get[Boolean]("enabled") ~
      get[String]("status") map {
         case recipient~enabled~status => {
            Relay(recipient,enabled,status)
         }
      }
   }

   def findRelaysForDomain(domain: Domain): List[Relay] = {
      domain.connection match {
         case Some(connection) => findRelaysForDomain(connection,domain)
         case None => throw new IllegalStateException("No connection in domain")
      }
   }

   def findRelaysForDomain(connection: ConnectionName, domain: Domain): List[Relay] = {
      DB.withConnection(connection) { implicit connection =>
         SQL(
            """
select * from relays
where recipient like {name}
order by recipient
            """
         ).on(
            'name -> s"%@${domain.name}"
         ).as(simpleRelay *)
      }
   }

   def findCatchAll(connection: ConnectionName, domain: Domain): Option[Relay] = findRelay(connection,s"@${domain.name}")

   def findRelay(alias: String, domain: Domain): Option[Relay] = findRelay(domain.connection.get, s"${alias}@${domain.name}")

   def disable(connection: ConnectionName, recipient: String) {
      DB.withConnection(connection) { implicit connection =>
         SQL(
            """
update relays set enabled = 0 where recipient = {recipient}
            """
         ).on(
            'recipient -> recipient
         ).executeUpdate
      }
   }

   def enable(connection: ConnectionName, recipient: String) {
      DB.withConnection(connection) { implicit connection =>
         SQL(
            """
update relays set enabled = 1 where recipient = {recipient}
            """
         ).on(
            'recipient -> recipient
         ).executeUpdate
      }
   }

   def findRelay(connection: ConnectionName, recipient: String): Option[Relay] = {
      DB.withConnection(connection) { implicit connection =>
         SQL(
            """
select * from relays
where recipient = {recipient}
order by recipient
            """
         ).on(
            'recipient -> recipient
         ).as(simpleRelay *).headOption
      }
   }

}

