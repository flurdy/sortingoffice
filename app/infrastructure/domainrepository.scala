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
      get[Int]("enabled") map {
         case domain~transport~enabled => {
            Domain(domain,enabled==1,transport)
         }
      }
   }

   private val domains: List[Domain] = List(
      Domain("example.no",true,"virtual"),
      Domain("example.de",false,"virtual"),
      Domain("example.it",true,"virtual")
   )

   private val backups: List[Domain] = List(
      Domain("example.se",true,"smtp:[mail.example.com]"),
      Domain("example.ru",false,"smtp:[mail.example.com]"),
      Domain("example.in",true,"smtp:[mail.example.com]")
   )

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

}

