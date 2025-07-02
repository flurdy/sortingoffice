package models

import infrastructure._
import scala.jdk.CollectionConverters._
import models.Environment.ConnectionName

case class Relay(recipient: String, enabled: Boolean, status: String) {

   def disable(connection: ConnectionName, featureToggles: FeatureToggles, relayRepository: RelayRepository) = {
      if(featureToggles.isToggleEnabled(connection)) relayRepository.disable(connection,this.recipient)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def enable(connection: ConnectionName, featureToggles: FeatureToggles, relayRepository: RelayRepository) = {
      if(featureToggles.isToggleEnabled(connection)) relayRepository.enable(connection,this.recipient)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def save(connection: ConnectionName, featureToggles: FeatureToggles, relayRepository: RelayRepository) = {
      if(featureToggles.isAddEnabled(connection)) relayRepository.save(connection,this)
      else throw new IllegalStateException("Add feature is disabled")
   }

   def delete(connection: ConnectionName, featureToggles: FeatureToggles, relayRepository: RelayRepository) = {
      if(featureToggles.isRemoveEnabled(connection)) relayRepository.delete(connection,this)
      else throw new IllegalStateException("Remove feature is disabled")
   }

   def reject(connection: ConnectionName, featureToggles: FeatureToggles, relayRepository: RelayRepository) = {
      if(featureToggles.isToggleEnabled(connection)) relayRepository.reject(connection,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def accept(connection: ConnectionName, featureToggles: FeatureToggles, relayRepository: RelayRepository) = {
      if(featureToggles.isToggleEnabled(connection)) relayRepository.accept(connection,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def isCatchAll = recipient.startsWith("@")

   def findInDatabases(environment: Environment, relays: Relays): List[(ConnectionName,Option[Relay])] = {
      for {
         connectionName <- environment.connectionNames
         relay          = relays.findRelay(connectionName,recipient)
      } yield (connectionName,relay)
   }

}

class Relays(featureToggles: FeatureToggles, aliases: Aliases, relayRepository: RelayRepository) {

   def findCatchAllDomainsIfEnabled(connection: ConnectionName): Option[List[(Domain,Relay)]] = {
      if( featureToggles.isRelayEnabled(connection) ){
         Some( relayRepository.findCatchAllRelayDomains(connection) )
      } else None
   }

   def findCatchAllDomainsIfEnabled(connection: ConnectionName,domains: List[Domain]): Option[(List[(Domain,Relay)],List[(Domain,Option[Relay])])] = {
      if( featureToggles.isRelayEnabled(connection) ){
         val catchAlls: List[(Domain,Relay)] = for{
            domain <- domains
            catchAll <- relayRepository.findCatchAll(connection,domain)
         } yield (domain,catchAll)
         val disabled: List[(Domain,Relay)] = catchAlls.filterNot(_._2.enabled)
         val disabledCatchAlls: List[(Domain,Option[Relay])] = disabled.map( c => (c._1,Some(c._2) ) )
         val nonCatchAllDomains: List[Domain] = domains diff catchAlls.map(_._1)
         val nonCatchAlls: List[(Domain,Option[Relay])] = nonCatchAllDomains.map( (_,None) ) ++ disabledCatchAlls
         Some((catchAlls,nonCatchAlls))
      } else None
   }

   def findCatchAllBackupsIfEnabled(connection: ConnectionName,domains: List[Domain]): Option[(List[(Backup,Relay)],List[(Backup,Option[Relay])])] = {
      if( featureToggles.isRelayEnabled(connection) ){
         findCatchAllDomainsIfEnabled(connection,domains).map( catchAlls =>
            (  catchAlls._1.map( c => (Backup(c._1),c._2) ),
               catchAlls._2.map( c => (Backup(c._1),c._2) ) ) )
      } else None
   }

   def findRequiredRelaysIfEnabled(domain: Domain): Option[Map[String,Relay]] = {
      if( featureToggles.isRelayEnabled(domain.connection.get) ){
         Some( findRelays(aliases.requiredAliases,domain) )
      } else None
   }

   def findCommonRelaysIfEnabled(domain: Domain): Option[Map[String,Relay]] = {
      if( featureToggles.isRelayEnabled(domain.connection.get) ){
         Some( findRelays(aliases.commonAliases,domain) )
      } else None
   }

   def findCustomRelaysIfEnabled(domain: Domain): Option[Map[String,Relay]] = {
      if( featureToggles.isRelayEnabled(domain.connection.get) ){
         Some( findRelays(aliases.customAliases,domain) )
      } else None
   }

   private def findRelays(aliasesToFind: List[String], domain: Domain): Map[String,Relay] = {
      ( for{
         aliasToFind <- aliasesToFind
         alias <- relayRepository.findRelay(aliasToFind,domain)
      } yield (aliasToFind,alias) ).toMap
   }

   def findRequiredAndCommonRelaysIfEnabled(connection: ConnectionName, domains: List[Domain]) : Option[List[(Domain,Map[String,Boolean])]]= {
      if( featureToggles.isRelayEnabled(connection) ){
         Some(
            for {
               domain <- domains
               requiredRelays <- findRequiredRelaysIfEnabled(domain)
               commonRelays <- findCommonRelaysIfEnabled(domain)
               relays = requiredRelays ++ commonRelays
            } yield ( domain, relays.map( relay => (relay._1, relay._2.enabled) ) )
         )
      } else None
   }

   def findRelay(connection: ConnectionName, recipient: String): Option[Relay] = relayRepository.findRelay(connection,recipient)

   def findOrphanRelays(connection: ConnectionName, domains: List[Domain]): List[Relay] = {
      val relays = relayRepository.findRelays(connection)
      val nonOrphans = for{
         relay <- relays
         domainName <- parseDomainName(relay)
         if domains.exists( _.name == domainName)
      } yield relay
      relays diff nonOrphans
   }

   private def parseDomainName(relay: Relay): Option[String] = Aliases.parseDomainName(relay.recipient)

   def findRelaysForAliasIfEnabled(connection: ConnectionName, domain: Domain, alias: Alias): Option[(Option[Relay],Option[Relay])] = {
      if( featureToggles.isRelayEnabled(connection) ){
         relayRepository.findRelay(connection, alias.mail) match {
            case Some(relay) if relay.isCatchAll => Some(None, Some(relay) )
            case Some(relay) => Some(relayRepository.findCatchAll(connection,domain), Some(relay) )
            case None        => Some(relayRepository.findCatchAll(connection,domain), None )
         }
      } else None
   }

}

object Relay {
  def unapply(r: Relay): Option[(String, Boolean, String)] = Some((r.recipient, r.enabled, r.status))
}
