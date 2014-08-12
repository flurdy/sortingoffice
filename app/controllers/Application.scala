package controllers

import play.api._
import play.api.mvc._
import models._
import models.Environment.ConnectionName

trait DbController extends Controller {

  implicit val databaseConnections: List[(String,String)] = Environment.databaseConnections

}

object Application extends DbController {

  def index = Action {
    Ok(views.html.connections(databaseConnections))
  }

  def connectionIndex(connection: ConnectionName) = Action {
    Ok(views.html.index(connection))
  }

  def about = Action {
    Ok(views.html.about())
  }

  def contact = Action {
    Ok(views.html.contact())
  }

}

object DomainController extends DbController {

  def domain(connection: ConnectionName) = Action {
    val relays = Domains.findRelayDomains(connection)
    val backups = Domains.findBackupDomains(connection)
    Ok(views.html.domain.domain( connection, relays, backups ))
  }

  def alias(connection: ConnectionName, name: String) = Action {
    Domains.findRelayDomain(connection, name) match {
      case Some(domain) =>{
        val relays = domain.findRelays
        val aliases = domain.findAliases
        val users = domain.findUsers
        Ok(views.html.domain.domainalias( connection, domain,relays,aliases,users))
      }
      case None => {
        Logger.warn(s"Domain $name not found")
        val relays = Domains.findRelayDomains(connection)
        val backups = Domains.findBackupDomains(connection)
        NotFound(views.html.domain.domain( connection, relays, backups ))
      }
    }
  }

}


object AliasController extends DbController {

  def alias(connection: ConnectionName) = Action {
    Ok(views.html.alias.alias(connection))
  }

  def catchAll(connection: ConnectionName) = Action {
    val catchAllAliases = Aliases.findCatchAllDomains(connection)
    val relayDomains = Domains.findRelayDomains(connection)
    val noCatchAllAliases = relayDomains diff catchAllAliases
    val catchAllRelays = Relays.findCatchAllDomains(connection)
    val noCatchAllRelays = relayDomains diff catchAllRelays
    Ok(views.html.alias.catchall(connection,catchAllAliases,noCatchAllAliases,catchAllRelays,noCatchAllRelays))
  }

  def common(connection: ConnectionName) = Action {
    val relayDomains = Domains.findRelayDomains(connection)
    val requiredAliases: List[(Domain,Map[String,Boolean])] = relayDomains.map{ d =>
      val aliases = d.findRequiredAliases ++ d.findCommonAliases
      ( d, aliases.map( a => (a._1,a._2.enabled) ) )
    }
    val requiredRelays:  List[(Domain,Map[String,Boolean])] = relayDomains.map{ d =>
      val relays = d.findRequiredRelays ++ d.findCommonRelays
      ( d, relays.map( r => (r._1,r._2.enabled) ) )
    }
    Ok(views.html.alias.common( connection, requiredAliases, requiredRelays ))
  }

  def crossDomain(connection: ConnectionName) = Action {
    val aliases = Aliases.customAliases
    val relayDomains = Domains.findRelayDomains(connection)
    val customAliases: List[(Domain,Map[String,Boolean],Map[String,Boolean])] = relayDomains.map{ d =>
      ( d, d.findCustomAliases.map( r => (r._1,r._2.enabled) ), d.findCustomRelays.map( r => (r._1,r._2.enabled) ) )
    }
    Ok(views.html.alias.cross(connection, aliases, customAliases) )
  }

}


object UserController extends DbController {

  def user(connection: ConnectionName) = Action {
    val users = Users.findUsers(connection)
    Ok(views.html.user.user(connection,users))
  }

  def edituser(connection: ConnectionName, email: String) = Action {
    Users.findUser(connection, email) match {
      case Some(user) => Ok(views.html.user.edituser(connection,user))
      case None => NotFound(s"No user known as [$email]")
    }
    
  }

}
