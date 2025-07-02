package models

import infrastructure._
import scala.jdk.CollectionConverters._
import models.Environment.ConnectionName

case class User(email: String, name: String, maildir: String, passwordReset: Boolean, enabled: Boolean) {

  def disable(connection: ConnectionName, featureToggles: FeatureToggles, userRepository: UserRepository) = {
    if(featureToggles.isToggleEnabled(connection)) userRepository.disable(connection,this)
    else throw new IllegalStateException("Toggle feature is disabled")
  }

  def enable(connection: ConnectionName, featureToggles: FeatureToggles, userRepository: UserRepository) = {
    if(featureToggles.isToggleEnabled(connection)) userRepository.enable(connection,this)
    else throw new IllegalStateException("Toggle feature is disabled")
  }

  def save(connection: ConnectionName, featureToggles: FeatureToggles, userRepository: UserRepository) = {
    if(featureToggles.isAddEnabled(connection)) userRepository.save(connection,this)
    else throw new IllegalStateException("Add feature is disabled")
  }

  def delete(connection: ConnectionName, featureToggles: FeatureToggles, userRepository: UserRepository) = {
    if(featureToggles.isRemoveEnabled(connection)) userRepository.delete(connection,this)
    else throw new IllegalStateException("Remove feature is disabled")
  }

  def resetPassword(connection: ConnectionName, featureToggles: FeatureToggles, userRepository: UserRepository) = {
    if(featureToggles.isEditEnabled(connection)) userRepository.resetPassword(connection,this)
    else throw new IllegalStateException("Edit feature is disabled")
  }

  def findAlias(connection: ConnectionName, aliasRepository: AliasRepository): Option[Alias] = aliasRepository.findAlias(connection, email)

  def findDomain(connection: ConnectionName, domains: Domains, aliases: Aliases): Option[Domain] =
    domains.extractAndFindDomain(connection, email, aliases)

  def update(connection: ConnectionName, featureToggles: FeatureToggles, userRepository: UserRepository) = {
    if(featureToggles.isEditEnabled(connection)) userRepository.update(connection,this)
    else throw new IllegalStateException("Edit feature is disabled")
  }

}

class Users(userRepository: UserRepository, aliasRepository: AliasRepository) {

  def findUsers(connection: ConnectionName): List[User] = userRepository.findUsers(connection)

  def findUser(connection: ConnectionName, email: String): Option[User] = userRepository.findUser(connection,email)

  def findUserByMaildir(connection: ConnectionName, maildir: String): Option[User] = userRepository.findUserByMaildir(connection,maildir)

  def findOrphanUsers(connection: ConnectionName, domains: List[Domain]): List[User] = {
    val users = userRepository.findUsers(connection)
    val nonOrphans = for{
      user <- users
      domainName <- parseDomainName(user)
      if domains.exists( _.name == domainName)
    } yield user
    users diff nonOrphans
  }

  private def parseDomainName(user: User): Option[String] = Aliases.parseDomainName(user.email)

}

object User {
  def unapply(u: User): Option[(String, String, String, Boolean, Boolean)] =
    Some((u.email, u.name, u.maildir, u.passwordReset, u.enabled))
}
