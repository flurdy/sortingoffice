package models

import infrastructure._
import play.api.Play
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName


case class User(email: String, passwordReset: Boolean, enabled: Boolean){

	def disable(connection: ConnectionName) = UserRepository.disable(connection,this)

	def enable(connection: ConnectionName) = UserRepository.enable(connection,this)

}

object Users {

   def findUsers(connection: ConnectionName): List[User] = UserRepository.findUsers(connection)

   def findUser(connection: ConnectionName, email: String): Option[User] = UserRepository.findUser(connection,email)

}

