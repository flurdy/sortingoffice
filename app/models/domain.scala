package models

import infrastructure._


case class Domain(name: String, enabled: Boolean, transport: String){

   def findRelays = RelayRepository.findRelaysForDomain(this)

   def findAliases = AliasRepository.findAliasesForDomain(this)

   def findUsers = UserRepository.findUsersForDomain(this)

}


object Domains {

   def findRelayDomain(name: String): Option[Domain] = DomainRepository.findRelayDomain(name)

   def findRelayDomains: List[Domain] = DomainRepository.findRelayDomains

   def findBackupDomains: List[Domain] = DomainRepository.findBackupDomains

}


case class Relay(recipient: String, enabled: Boolean, status: String)

object Relays {

   def findCatchAllDomains = DomainRepository.findCatchAllRelayDomains

}


case class Alias(mail: String, destination: String, enabled: Boolean)

object Aliases {

   def findCatchAllDomains = DomainRepository.findCatchAllDomains

}

case class User(email: String, passwordReset: Boolean, enabled: Boolean)

