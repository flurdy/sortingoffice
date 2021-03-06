package infrastructure

import play.api.Play.current
import play.api.Logger
import org.joda.time.DateTime
import play.api.db.DB
import anorm._
import anorm.SqlParser._
import models._
import models.Environment.ConnectionName


object DomainRepository {

   val simpleDomain = {
      get[String]("domain") ~
      get[String]("transport") ~
      get[Boolean]("enabled") map {
         case domain~transport~enabled => {
            new Domain(domain,enabled,transport)
         }
      }
   }

   val simpleBackup = {
      get[String]("domain") ~
      get[String]("transport") ~
      get[Boolean]("enabled") map {
         case domain~transport~enabled => {
            Backup(new Domain(domain,enabled,transport))
         }
      }
   }

   def findDomains(connectionName: ConnectionName): List[Domain] = {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
select * from domains order by domain
            """
         ).as(simpleDomain *).map( _.withConnection(connectionName) )
      }
   }

   def findBackupDomains(connectionName: ConnectionName): List[Backup] = {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
select * from backups order by domain
            """
         ).as(simpleBackup *).map( _.withConnection(connectionName) )
      }
   }

   def findDomain(connectionName: ConnectionName, name: String): Option[Domain] = {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
select * from domains
where domain = {name}
            """
         ).on(
            'name -> name
         ).as(simpleDomain *).map( _.withConnection(connectionName) ).headOption
      }
   }

   def findBackupDomain(connectionName: ConnectionName, name: String): Option[Backup] = {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
select * from backups
where domain = {name}
            """
         ).on(
            'name -> name
         ).as(simpleBackup *).map( _.withConnection(connectionName) ).headOption
      }
   }

   def findCatchAllDomains(connectionName: ConnectionName): List[(Domain,Alias)] = {
      val domains: List[Domain] = DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
select d.domain,d.transport,a.enabled from domains d
inner join aliases a
on concat('@',d.domain) = a.mail
order by d.domain
            """
         ).as(simpleDomain *).map( _.withConnection(connectionName) )
      }
      for{
         domain <- domains
         name = domain.name
         alias <- AliasRepository.findAlias(connectionName,s"@$name")
      } yield (domain,alias)
   }

   def findCatchAllRelayDomains(connectionName: ConnectionName): List[(Domain,Relay)] = {
      val domains: List[Domain] = DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
select d.domain,d.transport,r.enabled from domains d
inner join relays r
on concat('@',d.domain) = r.recipient
order by d.domain
            """
         ).as(simpleDomain *).map( _.withConnection(connectionName) )
      }
      for{
         domain <- domains
         name   =  domain.name
         relay  <- RelayRepository.findRelay(connectionName,s"@$name")
      } yield (domain,relay)
   }

   def disable(connectionName: ConnectionName, domain: Domain) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
               update domains set enabled = 0 where domain = {name}
            """
         ).on(
            'name -> domain.name
         ).executeUpdate
      }
   }

   def enable(connectionName: ConnectionName, domain: Domain) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
               update domains set enabled = 1 where domain = {name}
            """
         ).on(
            'name -> domain.name
         ).executeUpdate
      }
   }

   def disableBackup(connectionName: ConnectionName, backup: Backup) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
               update backups set enabled = 0 where domain = {name}
            """
         ).on(
            'name -> backup.domain.name
         ).executeUpdate
      }
   }

   def enableBackup(connectionName: ConnectionName, backup: Backup) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
               update backups set enabled = 1 where domain = {name}
            """
         ).on(
            'name -> backup.domain.name
         ).executeUpdate
      }
   }

   def save(connectionName: ConnectionName, domain: Domain) = {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
insert into domains (domain,enabled,transport) values ({name},{enabled},{transport})
            """
         ).on(
            'name -> domain.name,
            'enabled -> domain.enabled,
            'transport -> domain.transport
         ).execute
      }
   }

   def saveBackup(connectionName: ConnectionName, backup: Backup) = {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
insert into backups (domain,enabled,transport) values ({name},{enabled},{transport})
            """
         ).on(
            'name      -> backup.domain.name,
            'enabled   -> backup.domain.enabled,
            'transport -> backup.domain.transport
         ).execute
      }
   }

   def delete(connectionName: ConnectionName, domain: Domain) = {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
delete from domains where domain = {name}
            """
         ).on(
            'name -> domain.name
         ).execute
      }
   }


   def deleteBackup(connectionName: ConnectionName, backup: Backup) = {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
delete from backups where domain = {name}
            """
         ).on(
            'name -> backup.domain.name
         ).execute
      }
   }

   def updateBackup(connectionName: ConnectionName, backup: Backup) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
update backups set transport = {transport} where domain = {name}
            """
         ).on(
            'transport -> backup.domain.transport,
            'name -> backup.domain.name
         ).executeUpdate
      }
   }

}

