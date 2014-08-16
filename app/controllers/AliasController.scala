package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName


object AliasController extends Controller with DbController {

  def alias(connection: ConnectionName) = ConnectionAction(connection) {
    Ok(views.html.alias.alias(connection))
  }

  def catchAll(connection: ConnectionName) = ConnectionAction(connection) {
    val catchAllAliases = Aliases.findCatchAllDomains(connection)
    val relayDomains = Domains.findRelayDomains(connection)
    val noCatchAllAliases = relayDomains diff catchAllAliases
    val catchAllRelays: Option[List[Domain]] = Relays.findCatchAllDomainsIfEnabled(connection)
    val noCatchAllRelays: Option[List[Domain]] = catchAllRelays.map(relayDomains diff _)
    Ok(views.html.alias.catchall(connection,catchAllAliases,noCatchAllAliases,catchAllRelays,noCatchAllRelays))
  }

  def common(connection: ConnectionName) = ConnectionAction(connection) {
    val domains = Domains.findRelayDomains(connection)

    val requiredAliases: List[(Domain,Map[String,Boolean])] = Aliases.findRequiredAndCommonAliases(domains)

    val requiredRelays: Option[List[(Domain,Map[String,Boolean])]] = Relays.findRequiredAndCommonRelaysIfEnabled(connection,domains)
    /*
    val requiredAliases: List[(Domain,Map[String,Boolean])] = relayDomains.map{ d =>
      val aliases = d.findRequiredAliases ++ d.findCommonAliases
      ( d, aliases.map( a => (a._1,a._2.enabled) ) )
    }
    val requiredRelays:  List[(Domain,Map[String,Boolean])] = relayDomains.map{ d =>
      val relays = d.findRequiredRelays ++ d.findCommonRelays
      ( d, relays.map( r => (r._1,r._2.enabled) ) )
    }
    */
    Ok(views.html.alias.common( connection, requiredAliases, requiredRelays ))
  }

  def crossDomain(connection: ConnectionName) = ConnectionAction(connection) {
    val aliases = Aliases.customAliases
    val relayDomains = Domains.findRelayDomains(connection)

    val customAliases: List[(Domain, (Map[String,Boolean], Option[Map[String,Boolean]]))] = relayDomains.map{ d =>
      ( d, d.findCustomAliasesAndRelays )
    //  ( d, d.findCustomAliases.map( r => (r._1,r._2.enabled) ), d.findCustomRelays.map( r => (r._1,r._2.enabled) ) )
    }

    Ok(views.html.alias.cross(connection, aliases, customAliases) )
  }

}

