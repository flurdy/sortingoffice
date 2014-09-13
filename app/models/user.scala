package models

import infrastructure._
import play.api.Play
import play.api.Logger
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName


case class User(email: String, name: String, maildir: String, passwordReset: Boolean, enabled: Boolean){

	def disable(connection: ConnectionName) = {
      if(FeatureToggles.isToggleEnabled(connection)) UserRepository.disable(connection,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

	def enable(connection: ConnectionName) = {
      if(FeatureToggles.isToggleEnabled(connection)) UserRepository.enable(connection,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

	def save(connection: ConnectionName) = {
      if(FeatureToggles.isAddEnabled(connection)) UserRepository.save(connection,this)
      else throw new IllegalStateException("Add feature is disabled")
   }

   def delete(connection: ConnectionName) = {
      if(FeatureToggles.isRemoveEnabled(connection)) UserRepository.delete(connection,this)
      else throw new IllegalStateException("Remove feature is disabled")
   }

	def resetPassword(connection: ConnectionName) = {
      if(FeatureToggles.isEditEnabled(connection)) UserRepository.resetPassword(connection,this)
      else throw new IllegalStateException("Edit feature is disabled")
   }

   def findAlias(connection: ConnectionName): Option[Alias] = AliasRepository.findAlias(connection,email)
   
   def findDomain(connection: ConnectionName): Option[Domain] = Domains.extractAndFindDomain(connection,email)

   def update(connection: ConnectionName) = {
      if(FeatureToggles.isEditEnabled(connection)) UserRepository.update(connection,this)
      else throw new IllegalStateException("Edit feature is disabled")
   }

}

object Users {

   def findUsers(connection: ConnectionName): List[User] = UserRepository.findUsers(connection)

   def findUser(connection: ConnectionName, email: String): Option[User] = UserRepository.findUser(connection,email)

   def findUserByMaildir(connection: ConnectionName, maildir: String): Option[User] = UserRepository.findUserByMaildir(connection,maildir)

   def findOrphanUsers(connection: ConnectionName, domains: List[Domain]): List[User] = {
      val users = UserRepository.findUsers(connection)
      val nonOrphans = for{
         user <- users
         domainName <- parseDomainName(user)
         if domains.exists( _.name == domainName)
      } yield user
      users diff nonOrphans
   }

   private def parseDomainName(user: User): Option[String] = Aliases.parseDomainName(user.email)

}

