package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName


class RequestWithConnection[A](val connection: ConnectionName, request: Request[A]) extends WrappedRequest[A](request)

trait DbController {

  implicit val databaseConnections: List[(String,String)] = Environment.databaseConnections

  def isValidConnection(connection: ConnectionName): Boolean = databaseConnections.exists( _._1 == connection )

  def ConnectionAction(connection: String) = new ActionBuilder[RequestWithConnection] {
    def invokeBlock[A](request: Request[A], block: (RequestWithConnection[A]) => Future[SimpleResult]) = {
      if( isValidConnection(connection) ){
        block(new RequestWithConnection(connection, request))
      } else {        
        implicit val errorMessages = List(ErrorMessage("Connection not found"))
        Future.successful(
          NotFound(views.html.connections(databaseConnections))
        )
      }
    }
  }

}


object Application extends Controller with DbController {

  def index = Action {
    databaseConnections.size match {
      case 0 => NotFound(views.html.connections(List.empty))
      case 1 => Redirect(routes.Application.connectionIndex(databaseConnections.head._1))
      case _ => Ok(views.html.connections(databaseConnections))
    }           
  }

  def connectionIndex(connection: ConnectionName) = ConnectionAction(connection) {
      Ok(views.html.index(connection))
  }

  def about = Action {
    Ok(views.html.about())
  }

  def contact = Action {
    Ok(views.html.contact())
  }

}

object DomainController extends Controller with DbController {

  def domain(connection: ConnectionName) = ConnectionAction(connection) {
    val relays = Domains.findRelayDomains(connection)
    val backups = Domains.findBackupDomains(connection)
    Ok(views.html.domain.domain( connection, relays, backups ))
  }

  def alias(connection: ConnectionName, name: String) = ConnectionAction(connection) {
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
        implicit val errorMessages = List(ErrorMessage("Domain not found"))
        NotFound(views.html.domain.domain( connection, relays, backups))
      }
    }
  }   

}


object AliasController extends Controller with DbController {

  def alias(connection: ConnectionName) = ConnectionAction(connection) {
    Ok(views.html.alias.alias(connection))
  }

  def catchAll(connection: ConnectionName) = ConnectionAction(connection) {
    val catchAllAliases = Aliases.findCatchAllDomains(connection)
    val relayDomains = Domains.findRelayDomains(connection)
    val noCatchAllAliases = relayDomains diff catchAllAliases
    val catchAllRelays = Relays.findCatchAllDomains(connection)
    val noCatchAllRelays = relayDomains diff catchAllRelays
    Ok(views.html.alias.catchall(connection,catchAllAliases,noCatchAllAliases,catchAllRelays,noCatchAllRelays))
  }

  def common(connection: ConnectionName) = ConnectionAction(connection) {
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

  def crossDomain(connection: ConnectionName) = ConnectionAction(connection) {
    val aliases = Aliases.customAliases
    val relayDomains = Domains.findRelayDomains(connection)
    val customAliases: List[(Domain,Map[String,Boolean],Map[String,Boolean])] = relayDomains.map{ d =>
      ( d, d.findCustomAliases.map( r => (r._1,r._2.enabled) ), d.findCustomRelays.map( r => (r._1,r._2.enabled) ) )
    }
    Ok(views.html.alias.cross(connection, aliases, customAliases) )
  }

}


object UserController extends Controller with DbController {

  def user(connection: ConnectionName) = ConnectionAction(connection) {
    val users = Users.findUsers(connection)
    Ok(views.html.user.user(connection,users))
  }

  def edituser(connection: ConnectionName, email: String) = ConnectionAction(connection) {
    Users.findUser(connection, email) match {
      case Some(user) => Ok(views.html.user.edituser(connection,user))
      case None => {
        Logger.warn(s"User $email not found")
        val users = Users.findUsers(connection)
        implicit val errorMessages = List(ErrorMessage("User not found"))
        NotFound(views.html.user.user( connection, users))
      }
    }
  }

}
