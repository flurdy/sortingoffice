package models

import infrastructure._
import play.api.Play
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName


case class Alias(mail: String, destination: String, enabled: Boolean){

   def disable(connection: ConnectionName) = AliasRepository.disable(connection,mail)

   def enable(connection: ConnectionName) = AliasRepository.enable(connection,mail)

}

object Aliases {

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

}
