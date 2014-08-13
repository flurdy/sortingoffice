package models

import infrastructure._
import play.api.Play
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName



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
