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
    val toggle = FeatureToggles.isToggleEnabled(connection)
    Ok(views.html.alias.catchall(connection,catchAllAliases,noCatchAllAliases,catchAllRelays,noCatchAllRelays,toggle))
  }

  def common(connection: ConnectionName) = ConnectionAction(connection) {
    val domains = Domains.findRelayDomains(connection)
    val requiredAliases: List[(Domain,Map[String,Boolean])] = Aliases.findRequiredAndCommonAliases(domains)
    val requiredRelays: Option[List[(Domain,Map[String,Boolean])]] = Relays.findRequiredAndCommonRelaysIfEnabled(connection,domains)
    Ok(views.html.alias.common( connection, requiredAliases, requiredRelays ))
  }

  def crossDomain(connection: ConnectionName) = ConnectionAction(connection) {
    val aliases = Aliases.customAliases
    val relayDomains = Domains.findRelayDomains(connection)

    val customAliases: List[(Domain, (Map[String,Boolean], Option[Map[String,Boolean]]))] = relayDomains.map{ d =>
      ( d, d.findCustomAliasesAndRelays )
    }

    Ok(views.html.alias.cross(connection, aliases, customAliases) )
  }

  def disable(connection: ConnectionName, email: String) = ConnectionAction(connection) {
    Aliases.findAlias(connection,email) match {
      case Some(alias) => {
        alias.disable(connection)
        Redirect(routes.AliasController.alias(connection))
      }
      case None => {
        Logger.warn(s"Alias $email not found")
        implicit val errorMessages = List(ErrorMessage("Alias not found"))
        NotFound(views.html.alias.alias(connection))
      }
    }
  }

  def enable(connection: ConnectionName, email: String) = ConnectionAction(connection) {
    Aliases.findAlias(connection,email) match {
      case Some(alias) => {
        alias.enable(connection)
        Redirect(routes.AliasController.alias(connection))
      }
      case None => {
        Logger.warn(s"Alias $email not found")
        implicit val errorMessages = List(ErrorMessage("Alias not found"))
        NotFound(views.html.alias.alias(connection))
      }
    }
  }

}

