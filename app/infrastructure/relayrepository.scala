package infrastructure

import play.api.Play.current
import play.api.Logger
import org.joda.time.DateTime
import play.api.db.DB
import anorm._
import anorm.SqlParser._
import models._


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
      DB.withConnection { implicit connection =>
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

}


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

   def findAliasesForDomain(domain: Domain): List[Alias] = {
      DB.withConnection { implicit connection =>
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

}


object UserRepository {

   val simpleUser = {
      get[String]("id") ~
      get[String]("name") ~
      get[String]("maildir") ~
      get[Boolean]("change_password") ~
      get[Boolean]("enabled") map {
         case id~name~maildir~changePassword~enabled => {
            User(id, changePassword, enabled)
         }
      }
   }

   def findUsersForDomain(domain: Domain): List[User] = {
      DB.withConnection { implicit connection =>
         SQL(
            """
select * from users
where id like {name}
order by id
            """
         ).on(
            'name -> s"%@${domain.name}"
         ).as(simpleUser *)
      }
   }

}
