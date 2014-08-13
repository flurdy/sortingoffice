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


