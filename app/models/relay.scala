package models

import infrastructure._
import play.api.Play
import play.api.Logger
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName



case class Relay(recipient: String, enabled: Boolean, status: String){

   def disable(connection: ConnectionName) = {
      if(FeatureToggles.isToggleEnabled(connection)) RelayRepository.disable(connection,this.recipient)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def enable(connection: ConnectionName) = {
      if(FeatureToggles.isToggleEnabled(connection)) RelayRepository.enable(connection,this.recipient)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def save(connection: ConnectionName) = {
      if(FeatureToggles.isAddEnabled(connection)) RelayRepository.save(connection,this)
      else throw new IllegalStateException("Add feature is disabled")
   }

   def delete(connection: ConnectionName) = {
      if(FeatureToggles.isRemoveEnabled(connection)) RelayRepository.delete(connection,this)
      else throw new IllegalStateException("Remove feature is disabled")
   }

}

object Relays {

   def findCatchAllDomainsIfEnabled(connection: ConnectionName): Option[List[(Domain,Relay)]] = {
      if( FeatureToggles.isRelayEnabled(connection) ){
         Some( DomainRepository.findCatchAllRelayDomains(connection) )
      } else None
   }

   def findCatchAllDomainsIfEnabled(connection: ConnectionName,domains: List[Domain]): Option[(List[(Domain,Relay)],List[(Domain,Option[Relay])])] = {
      if( FeatureToggles.isRelayEnabled(connection) ){
         val catchAlls: List[(Domain,Relay)] = for{
            domain <- domains
            catchAll <- RelayRepository.findCatchAll(connection,domain)
         } yield (domain,catchAll)
         val disabled: List[(Domain,Relay)] = catchAlls.filterNot(_._2.enabled)
         val disabledCatchAlls: List[(Domain,Option[Relay])] = disabled.map( c => (c._1,Some(c._2) ) )
         val nonCatchAllDomains: List[Domain] = domains diff catchAlls.map(_._1)
         val nonCatchAlls: List[(Domain,Option[Relay])] = nonCatchAllDomains.map( (_,None) ) ++ disabledCatchAlls
         Some((catchAlls,nonCatchAlls))
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

   def findRelay(connection: ConnectionName, recipient: String): Option[Relay] = RelayRepository.findRelay(connection,recipient)

   def findOrphanRelays(connection: ConnectionName, domains: List[Domain]): List[Relay] = {
      val relays = RelayRepository.findRelays(connection)
      val nonOrphans = for{
         relay <- relays
         domainName <- parseDomainName(relay)
         if domains.exists( _.name == domainName)
      } yield relay
      relays diff nonOrphans
   }

   private def parseDomainName(relay: Relay): Option[String] = Aliases.parseDomainName(relay.recipient)

   def findRelaysForAliasIfEnabled(connection: ConnectionName, domain: Domain, alias: Alias): Option[(Option[Relay],Option[Relay])] = {
      if( FeatureToggles.isRelayEnabled(connection) ){  
         Some( RelayRepository.findCatchAll(connection,domain) , RelayRepository.findRelay(connection, alias.mail) )
      } else None
   }

}
