package infrastructure

import play.api.Play.current
import play.api.Logger
import org.joda.time.DateTime
import play.api.db.DB
import anorm._
import anorm.SqlParser._
import models._
import models.Environment.ConnectionName


object UserRepository {

   val simpleUser = {
      get[String]("id") ~
      get[String]("name") ~
      get[String]("maildir") ~
      get[Boolean]("change_password") ~
      get[Boolean]("enabled") map {
         case id~name~maildir~changePassword~enabled => {
            User(id, name, maildir, changePassword, enabled)
         }
      }
   }

   def findUsersForDomain(domain: Domain): List[User] = {
      DB.withConnection(domain.connection.get) { implicit connection =>
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

   def findUsers(connection: ConnectionName): List[User] = {
      DB.withConnection(connection) { implicit connection =>
         SQL(
            """
select * from users
order by id
            """
         ).as(simpleUser *)
      }
   }

   def findUser(connection: ConnectionName, email: String): Option[User] = {
      DB.withConnection(connection) { implicit connection =>
         SQL(
            """
select * from users
where id = {email}
order by id
            """
         ).on(
            'email -> email
         ).as(simpleUser *).headOption
      }
   }

   def findUserByMaildir(connection: ConnectionName, maildir: String): Option[User] = {
      DB.withConnection(connection) { implicit connection =>
         SQL(
            """
select * from users
where maildir = {maildir}
order by id
            """
         ).on(
            'maildir -> maildir
         ).as(simpleUser *).headOption
      }
   }

   def disable(connectionName: ConnectionName, user: User) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
               update users set enabled = 0 where id = {email}
            """
         ).on(
            'email -> user.email
         ).executeUpdate
      }
   }

   def enable(connectionName: ConnectionName, user: User) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
               update users set enabled = 1 where id = {email}
            """
         ).on(
            'email -> user.email
         ).executeUpdate
      }
   }

   def save(connectionName: ConnectionName, user: User) {
      DB.withConnection(connectionName) { implicit connection =>
         SQL(
            """
insert into users (id,name,maildir,change_password,enabled)
values ({email},{name},{maildir},{passwordReset},{enabled})
            """
         ).on(
            'email   -> user.email,
            'name    -> user.name,
            'maildir -> user.maildir,
            'passwordReset -> user.passwordReset,
            'enabled -> user.enabled
         ).execute
      }
   }


}
