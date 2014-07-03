package models


case class Domain(name: String, enabled: Boolean, transport: String)


object DomainRepository {

   private val domains: List[Domain] = List(
      Domain("example.no",true,"virtual"),
      Domain("example.de",false,"virtual"),
      Domain("example.it",true,"virtual")
   )

   private val backups: List[Domain] = List(
      Domain("example.se",true,"smtp:[mail.example.com]"),
      Domain("example.ru",false,"smtp:[mail.example.com]"),
      Domain("example.in",true,"smtp:[mail.example.com]")
   )

   def findRelayDomains: List[Domain] = domains

   def findBackupDomains: List[Domain] = backups

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
