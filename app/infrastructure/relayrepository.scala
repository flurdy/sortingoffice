package infrastructure

import org.joda.time.DateTime
import play.api.db.Database
import anorm._
import anorm.SqlParser._
import models._
import models.Environment.ConnectionName
import javax.inject._

@Singleton
class RelayRepository @Inject()(db: Database) {

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
      db.withConnection { implicit connection =>
         SQL(
            """
select * from relays
where recipient like {name}
order by recipient
            """
         ).on(
            "name" -> s"%@${domain.name}"
         ).as(simpleRelay.*)
      }
   }

   def findRelays(connection: ConnectionName): List[Relay] = {
      db.withConnection { implicit connection =>
         SQL(
            """
select * from relays
order by recipient
            """
         ).as(simpleRelay.*)
      }
   }

   def findCatchAll(connection: ConnectionName, domain: Domain): Option[Relay] = findRelay(connection,s"@${domain.name}")

   def findRelay(alias: String, domain: Domain): Option[Relay] = findRelay(domain.connection.get, s"${alias}@${domain.name}")

   def disable(connection: ConnectionName, recipient: String): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
update relays set enabled = 0 where recipient = {recipient}
            """
         ).on(
            "recipient" -> recipient
         ).executeUpdate()
      }
   }

   def enable(connection: ConnectionName, recipient: String): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
update relays set enabled = 1 where recipient = {recipient}
            """
         ).on(
            "recipient" -> recipient
         ).executeUpdate()
      }
   }

   def findRelay(connection: ConnectionName, recipient: String): Option[Relay] = {
      db.withConnection { implicit connection =>
         SQL(
            """
select * from relays
where recipient = {recipient}
order by recipient
            """
         ).on(
            "recipient" -> recipient
         ).as(simpleRelay.*).headOption
      }
   }

   def save(connectionName: ConnectionName, relay: Relay): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
insert into relays (recipient,status,enabled) values ({recipient},{status},{enabled})
            """
         ).on(
            "recipient" -> relay.recipient,
            "status" -> relay.status,
            "enabled" -> relay.enabled
         ).execute()
      }
   }

   def delete(connectionName: ConnectionName, relay: Relay): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
delete from relays where recipient = {recipient}
            """
         ).on(
            "recipient" -> relay.recipient
         ).execute()
      }
   }

   def reject(connection: ConnectionName, relay: Relay): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
update relays set status = 'REJECT' where recipient = {recipient}
            """
         ).on(
            "recipient" -> relay.recipient
         ).executeUpdate()
      }
   }

   def accept(connection: ConnectionName, relay: Relay): Unit = {
      db.withConnection { implicit connection =>
         SQL(
            """
update relays set status = 'OK' where recipient = {recipient}
            """
         ).on(
            "recipient" -> relay.recipient
         ).executeUpdate()
      }
   }

}
