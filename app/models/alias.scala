package models

import infrastructure._
import scala.collection.JavaConverters._
import models.Environment.ConnectionName


case class Alias(mail: String, destination: String, enabled: Boolean){

   def disable(connection: ConnectionName, featureToggles: FeatureToggles, aliasRepository: AliasRepository) = {
      if(featureToggles.isToggleEnabled(connection)) aliasRepository.disable(connection,mail)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def enable(connection: ConnectionName, featureToggles: FeatureToggles, aliasRepository: AliasRepository) = {
      if(featureToggles.isToggleEnabled(connection)) aliasRepository.enable(connection,mail)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def save(connection: ConnectionName, featureToggles: FeatureToggles, aliasRepository: AliasRepository) = {
      if(featureToggles.isAddEnabled(connection)) aliasRepository.save(connection,this)
      else throw new IllegalStateException("Add feature is disabled")
   }

   def delete(connection: ConnectionName, featureToggles: FeatureToggles, aliasRepository: AliasRepository) = {
      if(featureToggles.isRemoveEnabled(connection)) aliasRepository.delete(connection,this)
      else throw new IllegalStateException("Remove feature is disabled")
   }

   def update(connection: ConnectionName, featureToggles: FeatureToggles, aliasRepository: AliasRepository) = {
      if(featureToggles.isEditEnabled(connection)) aliasRepository.updateDestination(connection,this)
      else throw new IllegalStateException("Edit feature is disabled")
   }

   def parseDomainName: Option[String] = Aliases.parseDomainName(mail)

   def findInDatabases: List[(ConnectionName,Option[Alias])] = {
      for {
         connectionName <- Environment.connectionNames
         domainName     <- parseDomainName
         domain         <- Domains.findDomain(connectionName,domainName)
         alias          =  Aliases.findAlias(connectionName,mail)
      } yield (connectionName,alias)
   }

}

class Aliases(val customAliases: List[String], aliasRepository: AliasRepository, domainRepository: DomainRepository) {

   type Email = String

   val requiredAliases = List("","abuse","postmaster")

   val commonAliases = List("info","root","support","webmaster")

   def findCatchAllDomains(connection: ConnectionName): List[(Domain,Alias)] = domainRepository.findCatchAllDomains(connection)

   def findCatchAllDomains(connection: ConnectionName, domains: List[Domain]): (List[(Domain,Alias)],List[(Domain,Option[Alias])]) = {
      val catchAlls: List[(Domain,Alias)] = for{
         domain <- domains
         catchAll <- aliasRepository.findCatchAll(connection,domain)
      } yield (domain,catchAll)
      val disabled: List[(Domain,Alias)] = catchAlls.filterNot(_._2.enabled)
      val disabledCatchAlls: List[(Domain,Option[Alias])] = disabled.map( catchAll => (catchAll._1,Some(catchAll._2) ) )
      val nonCatchAllDomains: List[Domain]  = domains diff catchAlls.map(_._1)
      val nonCatchAlls: List[(Domain,Option[Alias])]  = nonCatchAllDomains.map( (_,None) ) ++ disabledCatchAlls
      (catchAlls, nonCatchAlls)
   }

   def findAllAliases(connection: ConnectionName): List[(Alias,Option[Domain])] = {
      val domains: Map[String,Domain] = domainRepository.findDomains(connection).map( d => (d.name,d) ).toMap
      val aliases = aliasRepository.findAliases(connection)
      aliases.map( alias => (alias, alias.parseDomainName.flatMap(domains.get(_)) ) )
   }

   def findRequiredAliases(domain: Domain): Map[String,Alias] = findAliases(requiredAliases,domain)

   def findCommonAliases(domain: Domain): Map[String,Alias] = findAliases(commonAliases,domain)

   def findCustomAliases(domain: Domain): Map[String,Alias] = findAliases(customAliases,domain)

   def findAliases(aliasesToFind: List[String], domain: Domain): Map[String,Alias] = {
      ( for{
         aliasToFind <- aliasesToFind
         alias <- aliasRepository.findDomainAlias(aliasToFind,domain)
      } yield (aliasToFind,alias) ).toMap
   }

   def findRequiredAndCommonAliases(domains: List[Domain]): List[(Domain,Map[String,Boolean])] = {
      domains map { domain =>
         val aliases: Map[String,Alias] = findRequiredAliases(domain) ++ findCommonAliases(domain)
         ( domain, aliases.map( alias => (alias._1, alias._2.enabled) ) )
      }
   }

   def findAlias(connection: ConnectionName, email: String): Option[Alias] = aliasRepository.findAlias(connection, email)

   def findOrphanAliases(connection: ConnectionName, domains: List[Domain]): List[Alias] = {
      val aliases = aliasRepository.findAliases(connection)
      val nonOrphans = for{
         alias <- aliases
         domainName <- Aliases.parseDomainName(alias)
         if domains.exists( _.name == domainName)
      } yield alias
      aliases diff nonOrphans
   }

}

object Aliases {
   private val EmailPattern = """.*@([\w\.-]+)$""".r

   private def parseDomainName(alias: Alias): Option[String] = parseDomainName(alias.mail)

   def parseDomainName(email: String): Option[String] = {
      email match {
         case EmailPattern(domain) => Some(domain)
         case _ => None
      }
   }

   def createAlias(email: String) = Alias(email,email,false)
}

object Alias {
  def unapply(a: Alias): Option[(String, String, Boolean)] = Some((a.mail, a.destination, a.enabled))
}
