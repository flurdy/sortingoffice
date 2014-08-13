package models

import infrastructure._
import play.api.Play
import play.api.Play.current
import scala.collection.JavaConverters._
import models.Environment.ConnectionName


case class Alias(mail: String, destination: String, enabled: Boolean)

object Aliases {

   val requiredAliases = List("","abuse","postmaster")

   val commonAliases = List("info","root","support","webmaster")

   val customAliases: List[String] = "" :: Play.configuration.getStringList("aliases.common.custom").map(_.asScala.toList).getOrElse(Nil)

   def findCatchAllDomains(connection: ConnectionName) = DomainRepository.findCatchAllDomains(connection)

   def findRequiredAliases(domain: Domain): Map[String,Alias] = findAliases(requiredAliases,domain)

   def findCommonAliases(domain: Domain): Map[String,Alias] = findAliases(commonAliases,domain)

   def findCustomAliases(domain: Domain): Map[String,Alias] = findAliases(customAliases,domain)

   def findAliases(aliasesToFind: List[String], domain: Domain): Map[String,Alias] = {
      ( for{
         aliasToFind <- aliasesToFind
         alias <- AliasRepository.findAlias(aliasToFind,domain)
      } yield (aliasToFind,alias) ).toMap
   }

}
