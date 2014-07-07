package controllers

import play.api._
import play.api.mvc._
import models._

object Application extends Controller {

  def index = Action {
    Ok(views.html.index())
  }

  def about = Action {
    Ok(views.html.about())
  }

  def contact = Action {
    Ok(views.html.contact())
  }

}


object DomainController extends Controller {

  def domain = Action {
    val relays = Domains.findRelayDomains
    val backups = Domains.findBackupDomains
    Ok(views.html.domain.domain( relays, backups ))
  }

  def alias(name: String) = Action {
    Domains.findRelayDomain(name) match {
      case Some(domain) =>{
        val relays = domain.findRelays
        val aliases = domain.findAliases
        val users = domain.findUsers
        Ok(views.html.domain.domainalias(domain,relays,aliases,users))
      }
      case None => {
        val relays = Domains.findRelayDomains
        val backups = Domains.findBackupDomains
        NotFound(views.html.domain.domain( relays, backups ))
      }
    }
  }

}


object AliasController extends Controller {

  def alias = Action {
    Ok(views.html.alias.alias())
  }

  def catchAll = Action {
    val catchAllAliases = Aliases.findCatchAllDomains
    val relayDomains = Domains.findRelayDomains
    val noCatchAllAliases = relayDomains diff catchAllAliases
    val catchAllRelays = Relays.findCatchAllDomains
    val noCatchAllRelays = relayDomains diff catchAllRelays
    Ok(views.html.alias.catchall(catchAllAliases,noCatchAllAliases,catchAllRelays,noCatchAllRelays))
  }

  def common = Action {
    val relayDomains = Domains.findRelayDomains
    val requiredAliases: List[(Domain,Map[String,Alias])] = relayDomains.map{ d =>
      ( d, d.findRequiredAliases ++ d.findCommonAliases )
    }
    val requiredRelays:  List[(Domain,Map[String,Alias])] = relayDomains.map{ d =>
      ( d, d.findRequiredRelays ++ d.findCommonRelays )
    }
    Ok(views.html.alias.common( requiredAliases, requiredRelays ))
  }

  def crossDomain = Action {
    Ok(views.html.alias.cross())
  }

}


object UserController extends Controller {

  def user = Action {
    Ok(views.html.user.user())
  }
  def edituser = Action {
    Ok(views.html.user.edituser())
  }

}
