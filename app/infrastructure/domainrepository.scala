package infrastructure

import play.api.Play.current
import play.api.Logger
import org.joda.time.DateTime
import play.api.db.DB
import anorm._
import anorm.SqlParser._
import models._
// import models.Environment.Datasource


object DomainRepository {

   val simpleDomain = {
      get[String]("domain") ~
      get[String]("transport") ~
      get[Boolean]("enabled") map {
         case domain~transport~enabled => {
            Domain(domain,enabled,transport)
         }
      }
   }


   val simpleBackup = {
      get[String]("domain") ~
      get[String]("transport") ~
      get[Boolean]("enabled") map {
         case domain~transport~enabled => {
            Domain(domain,enabled,transport)
         }
      }
   }

   def findRelayDomains: List[Domain] = {
      DB.withConnection { implicit connection =>
         SQL(
            """
select * from domains order by domain
            """
         ).as(simpleDomain *)
      }
   }

   def findBackupDomains: List[Domain] = {
      DB.withConnection { implicit connection =>
         SQL(
            """
select * from backups order by domain
            """
         ).as(simpleBackup *)
      }
   }

   def findRelayDomain(name: String): Option[Domain] = {
      DB.withConnection { implicit connection =>
         SQL(
            """
select * from domains
where domain = {name}
            """
         ).on(
            'name -> name
         ).as(simpleDomain *).headOption
      }
   }


   def findCatchAllDomains: List[Domain] = {
      DB.withConnection { implicit connection =>
         SQL(
            """
select d.domain,d.transport,a.enabled from domains d
inner join aliases a
on concat('@',d.domain) = a.mail
order by d.domain
            """
         ).as(simpleDomain *)
      }
   }

   def findCatchAllRelayDomains: List[Domain] = {
      DB.withConnection { implicit connection =>
         SQL(
            """
select d.domain,d.transport,r.enabled from domains d
inner join relays r
on concat('@',d.domain) = r.recipient
order by d.domain
            """
         ).as(simpleDomain *)
      }
   }



}

