package models

import infrastructure._
import play.api.Play
import play.api.Logger
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName


case class Alias(mail: String, destination: String, enabled: Boolean){

   def disable(connection: ConnectionName) = {
      if(FeatureToggles.isToggleEnabled(connection)) AliasRepository.disable(connection,mail)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def enable(connection: ConnectionName) = {
      if(FeatureToggles.isToggleEnabled(connection)) AliasRepository.enable(connection,mail)
      else throw new IllegalStateException("Toggle feature is disabled")
   }

   def save(connection: ConnectionName) = {
      if(FeatureToggles.isAddEnabled(connection)) AliasRepository.save(connection,this)
      else throw new IllegalStateException("Add feature is disabled")
   }

   def delete(connection: ConnectionName) = {
      if(FeatureToggles.isRemoveEnabled(connection)) AliasRepository.delete(connection,this)
      else throw new IllegalStateException("Remove feature is disabled")
   }

   def update(connection: ConnectionName) = {
      if(FeatureToggles.isEditEnabled(connection)) AliasRepository.updateDestination(connection,this)
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

object Aliases {

   type Email = String

   val requiredAliases = List("","abuse","postmaster")

   val commonAliases = List("info","root","support","webmaster")

   val customAliases: List[String] = "" :: Play.configuration.getStringList("aliases.common.custom").map(_.asScala.toList).getOrElse(Nil)

   def findCatchAllDomains(connection: ConnectionName): List[(Domain,Alias)] = DomainRepository.findCatchAllDomains(connection)

   def findCatchAllDomains(connection: ConnectionName, domains: List[Domain]): (List[(Domain,Alias)],List[(Domain,Option[Alias])]) = {
      val catchAlls: List[(Domain,Alias)] = for{
         domain <- domains
         catchAll <- AliasRepository.findCatchAll(connection,domain)
      } yield (domain,catchAll)
      val disabled: List[(Domain,Alias)] = catchAlls.filterNot(_._2.enabled)
      val disabledCatchAlls: List[(Domain,Option[Alias])] = disabled.map( catchAll => (catchAll._1,Some(catchAll._2) ) )
      val nonCatchAllDomains: List[Domain]  = domains diff catchAlls.map(_._1)
      val nonCatchAlls: List[(Domain,Option[Alias])]  = nonCatchAllDomains.map( (_,None) ) ++ disabledCatchAlls
      (catchAlls, nonCatchAlls)
   }

   def findAllAliases(connection: ConnectionName): List[(Alias,Option[Domain])] = {
      val domains: Map[String,Domain] = Domains.findDomains(connection).map( d => (d.name,d) ).toMap
      val aliases = AliasRepository.findAliases(connection)
      aliases.map( alias => (alias, alias.parseDomainName.flatMap(domains.get(_)) ) )
   }

   def findRequiredAliases(domain: Domain): Map[String,Alias] = findAliases(requiredAliases,domain)

   def findCommonAliases(domain: Domain): Map[String,Alias] = findAliases(commonAliases,domain)

   def findCustomAliases(domain: Domain): Map[String,Alias] = findAliases(customAliases,domain)

   def findAliases(aliasesToFind: List[String], domain: Domain): Map[String,Alias] = {
      ( for{
         aliasToFind <- aliasesToFind
         alias <- AliasRepository.findDomainAlias(aliasToFind,domain)
      } yield (aliasToFind,alias) ).toMap
   }

   def findRequiredAndCommonAliases(domains: List[Domain]): List[(Domain,Map[String,Boolean])] = {
      domains map { domain =>
         val aliases: Map[String,Alias] = domain.findRequiredAliases ++ domain.findCommonAliases
         ( domain, aliases.map( alias => (alias._1, alias._2.enabled) ) )
      }
   }

   def findAlias(connection: ConnectionName, email: String): Option[Alias] = AliasRepository.findAlias(connection, email)

   def findOrphanAliases(connection: ConnectionName, domains: List[Domain]): List[Alias] = {
      val aliases = AliasRepository.findAliases(connection)
      val nonOrphans = for{
         alias <- aliases
         domainName <- parseDomainName(alias)
         if domains.exists( _.name == domainName)
      } yield alias
      aliases diff nonOrphans
   }

   private val EmailPattern = """.*@([\w\.-]+)$""".r

   private def parseDomainName(alias: Alias): Option[String] = parseDomainName(alias.mail)

   def parseDomainName(email: Email): Option[String] = {
      email match {
         case EmailPattern(domain) => Some(domain)
         case _ => None
      }
   }

   def createAlias(email: Email) = Alias(email,email,false)

}
