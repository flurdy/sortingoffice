package models

import infrastructure._
import scala.jdk.CollectionConverters._
import models.Environment.ConnectionName

case class Domain(connection: Option[ConnectionName], name: String, enabled: Boolean, transport: String) {

   def this(name: String, enabled: Boolean, transport: String) = this(None, name, enabled, transport)

   def this(name: String) = this(None, name, false, ":[]")

   def withConnection(connection: ConnectionName) = Domain(Some(connection), name, enabled, transport)

   def findRelaysIfEnabled(featureToggles: FeatureToggles, domainRepository: DomainRepository) = {
      connection.flatMap { connectionName =>
         if( featureToggles.isRelayEnabled(connection.get)){
            Some( domainRepository.findRelaysForDomain(this) )
         } else None
      }
   }

   def findAliases(domainRepository: DomainRepository): List[Alias] = connection match {
      case Some(conn) => domainRepository.findAllAliasesForDomain(this)
      case None => throw new IllegalStateException("No connection for domain")
   }

   def findUsers(domainRepository: DomainRepository) = connection match {
      case Some(conn) => domainRepository.findUsersForDomain(this)
      case None => throw new IllegalStateException("No connection for domain")
   }

   def findRequiredAliases(aliases: Aliases): Map[String,Alias] = aliases.findRequiredAliases(this)

   def findCommonAliases(aliases: Aliases): Map[String,Alias] = aliases.findCommonAliases(this)

   def findRequiredRelaysIfEnabled(relays: Relays, featureToggles: FeatureToggles): Option[Map[String,Relay]] = relays.findRequiredRelaysIfEnabled(this, featureToggles)

   def findCommonRelaysIfEnabled(relays: Relays, featureToggles: FeatureToggles): Option[Map[String,Relay]] = relays.findCommonRelaysIfEnabled(this, featureToggles)

   def findCustomAliases(aliases: Aliases): Map[String,Alias] = aliases.findCustomAliases(this)

   def findCustomRelaysIfEnabled(relays: Relays, featureToggles: FeatureToggles): Option[Map[String,Relay]] = relays.findCustomRelaysIfEnabled(this, featureToggles)

   def findCustomAliasesAndRelays(aliases: Aliases, relays: Relays, featureToggles: FeatureToggles): (Map[String,Boolean],Option[Map[String,Boolean]]) = {
      ( findCustomAliases(aliases).map( a=> (a._1,a._2.enabled) ),
        findCustomRelaysIfEnabled(relays, featureToggles).map( r => r.map( re => (re._1,re._2.enabled) ) ) )
   }

   def disable(featureToggles: FeatureToggles, domainRepository: DomainRepository) = connection.map{ con =>
      if(featureToggles.isToggleEnabled(con)) domainRepository.disable(con,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def enable(featureToggles: FeatureToggles, domainRepository: DomainRepository) = connection.map{ con =>
      if(featureToggles.isToggleEnabled(con)) domainRepository.enable(con,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def save(featureToggles: FeatureToggles, domainRepository: DomainRepository) = connection.map{ con =>
      if(featureToggles.isAddEnabled(con)) domainRepository.save(con,this)
      else throw new IllegalStateException("Add feature is disabled")
   }

   def delete(featureToggles: FeatureToggles, domainRepository: DomainRepository) = connection.map{ con =>
      if(featureToggles.isRemoveEnabled(con)) domainRepository.delete(con,this)
      else throw new IllegalStateException("Remove feature is disabled")
   }

   def findInDatabases(environment: Environment, domains: Domains): List[(ConnectionName,Option[Domain],Option[Backup])] = {
      val domains: List[(ConnectionName,Option[Domain],Option[Backup])] = for {
         connectionName <- environment.connectionNames
         domain         = domains.findDomain(connectionName,name)
         backup         = domains.findBackupDomain(connectionName,name)
      } yield (connectionName,domain,backup)
      domains map {
         case (connectionName: ConnectionName, Some(domain), backup ) => (connectionName, Some(domain), backup )
         case (connectionName: ConnectionName, None, Some(backup) ) => (connectionName, Some(backup.domain), Some(backup) )
         case (connectionName: ConnectionName, domain, backup ) => (connectionName, domain, backup )
      }
   }

   def convertToBackup(featureToggles: FeatureToggles, domainRepository: DomainRepository): Option[Backup] = connection.map{ con =>
      if(featureToggles.isEditEnabled(con)){
         val newBackup = Backup(this.copy(transport = ":[]")).save(featureToggles, domainRepository)
         this.delete(featureToggles, domainRepository)
         newBackup
      } else throw new IllegalStateException("Edit feature is disabled")
   }

}


case class Backup(domain: Domain){

   def withConnection(connection: ConnectionName) = Backup( domain.withConnection(connection) )

   def enable(featureToggles: FeatureToggles, domainRepository: DomainRepository) = domain.connection.map{ con =>
      if(featureToggles.isToggleEnabled(con)) domainRepository.enableBackup(con,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def disable(featureToggles: FeatureToggles, domainRepository: DomainRepository) = domain.connection.map{ con =>
      if(featureToggles.isToggleEnabled(con)) domainRepository.disableBackup(con,this)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def save(featureToggles: FeatureToggles, domainRepository: DomainRepository) = domain.connection.map{ con =>
      if(featureToggles.isAddEnabled(con)) domainRepository.saveBackup(con,this)
      else throw new IllegalStateException("Add feature is disabled")
   }

   def delete(featureToggles: FeatureToggles, domainRepository: DomainRepository) = domain.connection.map{ con =>
      if(featureToggles.isRemoveEnabled(con)) domainRepository.deleteBackup(con,this)
      else throw new IllegalStateException("Remove feature is disabled")
   }

   def update(featureToggles: FeatureToggles, domainRepository: DomainRepository) = domain.connection.map{ con =>
      if(featureToggles.isEditEnabled(con)) domainRepository.updateBackup(con,this)
      else throw new IllegalStateException("Edit feature is disabled")
   }

   def convertToRelay(featureToggles: FeatureToggles, domainRepository: DomainRepository) = domain.connection.map{ con =>
      if(featureToggles.isEditEnabled(con)){
         val newDomain = domain.copy(transport="virtual:").save(featureToggles, domainRepository)
         this.delete(featureToggles, domainRepository)
         newDomain
      } else throw new IllegalStateException("Edit feature is disabled")
   }

}


class Domains(domainRepository: DomainRepository) {

   def findDomain(connection: ConnectionName, name: String): Option[Domain] = domainRepository.findDomain(connection, name)

   def findDomains(connection: ConnectionName): List[Domain] = domainRepository.findDomains(connection)

   def newDomain(connection: ConnectionName, name: String): Domain = Domain(Some(connection),name,false,"virtual:")

   def extractAndFindDomain(connection: ConnectionName, email: String, aliases: Aliases): Option[Domain] = {
      aliases.getClass.getMethod("parseDomainName", classOf[String]).invoke(aliases, email).asInstanceOf[Option[String]].flatMap( name => findDomain(connection,name) )
   }

   def findBackupDomain(connection: ConnectionName, name: String): Option[Backup] = domainRepository.findBackupDomain(connection, name)

   def findBackupDomains(connection: ConnectionName): List[Backup] = domainRepository.findBackupDomains(connection)

   def findBackupDomainsIfEnabled(connection: ConnectionName, featureToggles: FeatureToggles): Option[List[Backup]] = {
      if( featureToggles.isBackupEnabled(connection)){
         Some( domainRepository.findBackupDomains(connection) )
      } else None
   }

}

object Domain {
  def unapply(d: Domain): Option[(Option[Environment.ConnectionName], String, Boolean, String)] =
    Some((d.connection, d.name, d.enabled, d.transport))
}
