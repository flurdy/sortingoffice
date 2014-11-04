package models

import infrastructure._
import play.api.Play
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName



case class Domain(connection: Option[ConnectionName], name: String, enabled: Boolean, transport: String){

   def this(name: String, enabled: Boolean, transport: String) = this( None, name, enabled, transport)

   def this(name: String) = this( None, name, false, ":[]")

   def withConnection(connection: ConnectionName) = Domain( Some(connection), name, enabled, transport)

   def findRelaysIfEnabled = {
      connection flatMap { connectionName =>
         if( FeatureToggles.isRelayEnabled(connection.get)){
            Some( RelayRepository.findRelaysForDomain(this) )
         } else None
      }
   }

   def findAliases: List[Alias] = connection match {
      case Some(conn) => AliasRepository.findAllAliasesForDomain(this)
      case None => throw new IllegalStateException("No connection for domain")
   }

   def findUsers = connection match {
      case Some(conn) => UserRepository.findUsersForDomain(this)
      case None => throw new IllegalStateException("No connection for domain")
   }

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

   def disable = connection.map{ con =>
      if(FeatureToggles.isToggleEnabled(con)) DomainRepository.disable(con,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def enable = connection.map{ con =>
      if(FeatureToggles.isToggleEnabled(con)) DomainRepository.enable(con,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def save = connection.map{ con =>
      if(FeatureToggles.isAddEnabled(con)) DomainRepository.save(con,this)
      else throw new IllegalStateException("Add feature is disabled")
   }

   def delete = connection.map{ con =>
      if(FeatureToggles.isRemoveEnabled(con)) DomainRepository.delete(con,this)
      else throw new IllegalStateException("Remove feature is disabled")
   }

   def findInDatabases: List[(ConnectionName,Option[Domain],Option[Backup])] = {
      val domains: List[(ConnectionName,Option[Domain],Option[Backup])] = for {
         connectionName <- Environment.connectionNames
         domain         = Domains.findDomain(connectionName,name)
         backup         = Domains.findBackupDomain(connectionName,name)
      } yield (connectionName,domain,backup) 
      domains map {
         case (connectionName: ConnectionName, Some(domain), backup ) => (connectionName, Some(domain), backup )
         case (connectionName: ConnectionName, None, Some(backup) ) => (connectionName, Some(backup.domain), Some(backup) )
         case (connectionName: ConnectionName, domain, backup ) => (connectionName, domain, backup )
      } 
   }

   def convertToBackup = connection.map{ con =>
      if(FeatureToggles.isEditEnabled(con)){
         val newBackup = Backup(this.copy(transport = ":[]")).save
         this.delete
         newBackup
      } else throw new IllegalStateException("Edit feature is disabled")
   }

}


case class Backup(domain: Domain){

   def withConnection(connection: ConnectionName) = Backup( domain.withConnection(connection) )

   def enable = domain.connection.map{ con =>
      if(FeatureToggles.isToggleEnabled(con)) DomainRepository.enableBackup(con,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def disable = domain.connection.map{ con =>
      if(FeatureToggles.isToggleEnabled(con)) DomainRepository.disableBackup(con,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def save = domain.connection.map{ con =>
      if(FeatureToggles.isAddEnabled(con)) DomainRepository.saveBackup(con,this)
      else throw new IllegalStateException("Add feature is disabled")
   }

   def delete = domain.connection.map{ con =>
      if(FeatureToggles.isRemoveEnabled(con)) DomainRepository.deleteBackup(con,this)
      else throw new IllegalStateException("Remove feature is disabled")
   }

   def update = domain.connection.map{ con =>
      if(FeatureToggles.isEditEnabled(con)) DomainRepository.updateBackup(con,this)
      else throw new IllegalStateException("Edit feature is disabled")
   }

   def convertToRelay = domain.connection.map{ con =>
      if(FeatureToggles.isEditEnabled(con)){
         val newDomain = domain.copy(transport="virtual:").save
         this.delete
         newDomain
      } else throw new IllegalStateException("Edit feature is disabled")
   }
   
}


object Domains {

   def findDomain(connection: ConnectionName, name: String): Option[Domain] = DomainRepository.findDomain(connection, name)

   def findDomains(connection: ConnectionName): List[Domain] = DomainRepository.findDomains(connection)

   def newDomain(connection: ConnectionName, name: String): Domain = Domain(Some(connection),name,false,"virtual:")

   def extractAndFindDomain(connection: ConnectionName, email: String): Option[Domain] = {
      Aliases.parseDomainName(email).flatMap( name => findDomain(connection,name) )
   }

// }
// object Backups {

   def findBackupDomain(connection: ConnectionName, name: String): Option[Backup] = DomainRepository.findBackupDomain(connection, name)

   def findBackupDomains(connection: ConnectionName): List[Backup] = DomainRepository.findBackupDomains(connection)

   def findBackupDomainsIfEnabled(connection: ConnectionName): Option[List[Backup]] = {
      if( FeatureToggles.isBackupEnabled(connection)){
         Some( DomainRepository.findBackupDomains(connection) )
      } else None
   }

}


