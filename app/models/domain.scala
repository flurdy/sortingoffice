package models

import infrastructure._


case class Domain(name: String, enabled: Boolean, transport: String){

   def findRelays = RelayRepository.findRelaysForDomain(this)

   def findAliases = AliasRepository.findAllAliasesForDomain(this)

   def findUsers = UserRepository.findUsersForDomain(this)

   def findRequiredAliases: Map[String,Alias] = Aliases.findRequiredAliases(this)

   def findCommonAliases  : Map[String,Alias] = Aliases.findCommonAliases(this)

   def findRequiredRelays: Map[String,Alias] = Relays.findRequiredRelays(this)

   def findCommonRelays: Map[String,Alias] = Relays.findCommonRelays(this)

}


object Domains {

   def findRelayDomain(name: String): Option[Domain] = DomainRepository.findRelayDomain(name)

   def findRelayDomains: List[Domain] = DomainRepository.findRelayDomains

   def findBackupDomains: List[Domain] = DomainRepository.findBackupDomains

}


case class Relay(recipient: String, enabled: Boolean, status: String)

object Relays {

   def findCatchAllDomains = DomainRepository.findCatchAllRelayDomains

   def findRequiredRelays(domain: Domain): Map[String,Alias] = findRelays(Aliases.requiredAliases,domain)

   def findCommonRelays(domain: Domain): Map[String,Alias] = findRelays(Aliases.commonAliases,domain)

   def findRelays(aliasesToFind: List[String], domain: Domain): Map[String,Alias] = {
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

   def findCatchAllDomains = DomainRepository.findCatchAllDomains

   def findRequiredAliases(domain: Domain): Map[String,Alias] = findAliases(requiredAliases,domain)

   def findCommonAliases(domain: Domain): Map[String,Alias] = findAliases(commonAliases,domain)

   def findAliases(aliasesToFind: List[String], domain: Domain): Map[String,Alias] = {
      ( for{
         aliasToFind <- aliasesToFind
         alias <- AliasRepository.findAlias(aliasToFind,domain)
      } yield (aliasToFind,alias) ).toMap
   }

}

case class User(email: String, passwordReset: Boolean, enabled: Boolean)

