package models

import infrastructure._
import play.api.Play
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName


case class Domain(connection: Option[ConnectionName], name: String, enabled: Boolean, transport: String){

   def this(name: String, enabled: Boolean, transport: String) = this( None, name, enabled, transport)

   def withConnection(connection: ConnectionName) = Domain( Some(connection), name, enabled, transport)

   def findRelays = RelayRepository.findRelaysForDomain(this)

   def findAliases = AliasRepository.findAllAliasesForDomain(this)

   def findUsers = UserRepository.findUsersForDomain(this)

   def findRequiredAliases: Map[String,Alias] = Aliases.findRequiredAliases(this)

   def findCommonAliases  : Map[String,Alias] = Aliases.findCommonAliases(this)

   def findRequiredRelays: Map[String,Relay] = Relays.findRequiredRelays(this)

   def findCommonRelays: Map[String,Relay] = Relays.findCommonRelays(this)

   def findCustomAliases  : Map[String,Alias] = Aliases.findCustomAliases(this)

   def findCustomRelays: Map[String,Relay] = Relays.findCustomRelays(this)

}


object Domains {

   def findRelayDomain(connection: ConnectionName, name: String): Option[Domain] = DomainRepository.findRelayDomain(connection, name)

   def findRelayDomains(connection: ConnectionName): List[Domain] = DomainRepository.findRelayDomains(connection)

   def findBackupDomains(connection: ConnectionName): List[Domain] = DomainRepository.findBackupDomains(connection)

}


case class Relay(recipient: String, enabled: Boolean, status: String)

object Relays {

   def findCatchAllDomains(connection: ConnectionName) = DomainRepository.findCatchAllRelayDomains(connection)

   def findRequiredRelays(domain: Domain): Map[String,Relay] = findRelays(Aliases.requiredAliases,domain)

   def findCommonRelays(domain: Domain): Map[String,Relay] = findRelays(Aliases.commonAliases,domain)

   def findCustomRelays(domain: Domain): Map[String,Relay] = findRelays(Aliases.customAliases,domain)

   def findRelays(aliasesToFind: List[String], domain: Domain): Map[String,Relay] = {
      ( for{
         aliasToFind <- aliasesToFind
         alias <- RelayRepository.findRelay(aliasToFind,domain)
      } yield (aliasToFind,alias) ).toMap
   }

}


case class Alias(mail: String, destination: String, enabled: Boolean)

object Aliases {

   val requiredAliases = List("","abuse","postmaster")

   val commonAliases = List("info","root","support","webmaster")

   val customAliases: List[String] = "" :: Play.configuration.getStringList("aliases.common.custom").map(_.asScala.toList).getOrElse(Nil)

   def findCatchAllDomains(connection: ConnectionName) = DomainRepository.findCatchAllDomains(connection)

   def findRequiredAliases(domain: Domain): Map[String,Alias] = findAliases(requiredAliases,domain)

   def findCommonAliases(domain: Domain): Map[String,Alias] = findAliases(commonAliases,domain)

   def findCustomAliases(domain: Domain): Map[String,Alias] = findAliases(customAliases,domain)

   def findAliases(aliasesToFind: List[String], domain: Domain): Map[String,Alias] = {
      ( for{
         aliasToFind <- aliasesToFind
         alias <- AliasRepository.findAlias(aliasToFind,domain)
      } yield (aliasToFind,alias) ).toMap
   }

}


case class User(email: String, passwordReset: Boolean, enabled: Boolean)

object Users {

   def findUsers(connection: ConnectionName): List[User] = UserRepository.findUsers(connection)
   
   def findUser(connection: ConnectionName, email: String): Option[User] = UserRepository.findUser(connection,email)

}

