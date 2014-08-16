package models

import infrastructure._
import play.api.Play
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName



case class Relay(recipient: String, enabled: Boolean, status: String)

object Relays {

   def findCatchAllDomainsIfEnabled(connection: ConnectionName): Option[List[Domain]] = {
      if( FeatureToggles.isRelayEnabled(connection) ){
         Some( DomainRepository.findCatchAllRelayDomains(connection) )
      } else None
   }

   def findRequiredRelaysIfEnabled(domain: Domain): Option[Map[String,Relay]] = {
      if( FeatureToggles.isRelayEnabled(domain.connection.get) ){
         Some( findRelays(Aliases.requiredAliases,domain) )
      } else None
   }

   def findCommonRelaysIfEnabled(domain: Domain): Option[Map[String,Relay]] = {
      if( FeatureToggles.isRelayEnabled(domain.connection.get) ){
         Some( findRelays(Aliases.commonAliases,domain) )
      } else None
   }

   def findCustomRelaysIfEnabled(domain: Domain): Option[Map[String,Relay]] = {
      if( FeatureToggles.isRelayEnabled(domain.connection.get) ){
         Some( findRelays(Aliases.customAliases,domain) )
      } else None
   }

   private def findRelays(aliasesToFind: List[String], domain: Domain): Map[String,Relay] = {
      ( for{
         aliasToFind <- aliasesToFind
         alias <- RelayRepository.findRelay(aliasToFind,domain)
      } yield (aliasToFind,alias) ).toMap
   }

   def findRequiredAndCommonRelaysIfEnabled(connection: ConnectionName, domains: List[Domain]) : Option[List[(Domain,Map[String,Boolean])]]= {
      if( FeatureToggles.isRelayEnabled(connection) ){
         Some(
            for {
               domain <- domains
               requiredRelays <- domain.findRequiredRelaysIfEnabled
               commonRelays <- domain.findCommonRelaysIfEnabled
               relays = requiredRelays ++ commonRelays
            } yield ( domain, relays.map( relay => (relay._1, relay._2.enabled) ) )
         )
      } else None
   }

}
