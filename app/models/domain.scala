package models

import infrastructure._
import play.api.Play
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName



case class Domain(connection: Option[ConnectionName], name: String, enabled: Boolean, transport: String){

   def this(name: String, enabled: Boolean, transport: String) = this( None, name, enabled, transport)

   def withConnection(connection: ConnectionName) = Domain( Some(connection), name, enabled, transport)

   def findRelaysIfEnabled = {
      connection flatMap { connectionName =>
         if( FeatureToggles.isRelayEnabled(connection.get)){
            Some( RelayRepository.findRelaysForDomain(this) )
         } else None
      }
   }

   def findAliases = AliasRepository.findAllAliasesForDomain(this)

   def findUsers = UserRepository.findUsersForDomain(this)

   def findRequiredAliases: Map[String,Alias] = Aliases.findRequiredAliases(this)

   def findCommonAliases  : Map[String,Alias] = Aliases.findCommonAliases(this)

   def findRequiredRelaysIfEnabled: Option[Map[String,Relay]] = Relays.findRequiredRelaysIfEnabled(this)

   def findCommonRelaysIfEnabled: Option[Map[String,Relay]] = Relays.findCommonRelaysIfEnabled(this)

   def findCustomAliases  : Map[String,Alias] = Aliases.findCustomAliases(this)

   def findCustomRelaysIfEnabled: Option[Map[String,Relay]] = Relays.findCustomRelaysIfEnabled(this)

   def findCustomAliasesAndRelays: (Map[String,Boolean],Option[Map[String,Boolean]]) = {
      ( findCustomAliases.map( a=> (a._1,a._2.enabled) ),
        findCustomRelaysIfEnabled.map( r => r.map( re => (re._1,re._2.enabled) ) ) )
   }

   def disable = connection.map( Domains.disable(_,this) )

   def enable = connection.map( Domains.enable(_,this) )

}


object Domains {

   def findRelayDomain(connection: ConnectionName, name: String): Option[Domain] = DomainRepository.findRelayDomain(connection, name)

   def findRelayDomains(connection: ConnectionName): List[Domain] = DomainRepository.findRelayDomains(connection)

   def findBackupDomains(connection: ConnectionName): List[Domain] = DomainRepository.findBackupDomains(connection)

   def findBackupDomainsIfEnabled(connection: ConnectionName): Option[List[Domain]] = {
      if( FeatureToggles.isBackupEnabled(connection)){
         Some( DomainRepository.findBackupDomains(connection) )
      } else None
   }

   def disable(connection: ConnectionName, domain: Domain) {
      DomainRepository.disable(connection, domain)
   }

   def enable(connection: ConnectionName, domain: Domain) {
      DomainRepository.enable(connection, domain)
   }

}


