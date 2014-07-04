package models

import infrastructure._


case class Domain(name: String, enabled: Boolean, transport: String)


object Domains {

   def findRelayDomains: List[Domain] = DomainRepository.findRelayDomains

   def findBackupDomains: List[Domain] = DomainRepository.findBackupDomains

}


case class Relay(recipient: String, enabled: Boolean, transport: String)

case class Alias(mail: String, destination: String, enabled: Boolean)

case class User(email: String, passwordReset: Boolean, enabled: Boolean)

object RelayRepository {

   private val relays: List[Relay] = List(
      Relay("@example.com",true,"smtp")
   )

   def findRelaysForDomain(domain: Domain): List[Relay] = relays

}

object AliasRepository {

   private val aliases: List[Alias] = List(
      Alias("@example.com","john@example.io",true)
   )

   def findAliasesForDomain(domain: Domain): List[Alias] = aliases

}

object UserRepository {

   private val users: List[User] = List(
      User("john@example.com", true, true),
      User("mark@example.org", false, true)
   )

   def findUsersForDomain(domain: Domain): List[User] = users

}
