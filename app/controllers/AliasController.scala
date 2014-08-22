package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName


class RequestWithAlias[A](val alias: Alias, request: Request[A]) extends WrappedRequest[A](request)

trait AliasInjector {

  def AliasAction(email: String) = new ActionBuilder[RequestWithAlias] {
    def invokeBlock[A](request: Request[A], block: (RequestWithAlias[A]) => Future[SimpleResult]) = {      
      request match { 
        case connectionRequest: RequestWithConnection[A] => {
          Aliases.findAlias(connectionRequest.connection, email) match {
            case Some(alias) => {
              block(new RequestWithAlias(alias, connectionRequest))
            }
            case None => {
              Logger.warn(s"Alias $email not found")
              implicit val errorMessages = List(ErrorMessage("Alias not found")) 
              Future.successful( NotFound(views.html.alias.alias(connectionRequest.connection)(errorMessages) ) )
            }
          }          
        }
        case _ => Future.successful(InternalServerError)
      }
    }
  }

}

object AliasController extends Controller with DbController with FeatureToggler with AliasInjector {

  def alias(connection: ConnectionName) = ConnectionAction(connection) {
    Ok(views.html.alias.alias(connection))
  }

  def catchAll(connection: ConnectionName) = ConnectionAction(connection) { implicit request =>
    val allDomains: List[Domain] = Domains.findDomains(connection)
    val catchAllAliases: (List[(Domain,Alias)],List[(Domain,Option[Alias])]) = Aliases.findCatchAllDomains(connection,allDomains)
    val allBackups = Domains.findBackupDomains(connection)
    val catchAllRelays: Option[(List[(Domain,Relay)],List[(Domain,Option[Relay])])] = Relays.findCatchAllDomainsIfEnabled(connection,allDomains++allBackups)
    Ok(views.html.alias.catchall(
      connection,catchAllAliases._1,catchAllAliases._2,
      catchAllRelays.map(_._1),catchAllRelays.map(_._2) ) )
  }

  def common(connection: ConnectionName) = ConnectionAction(connection) { implicit request =>
    val domains = Domains.findDomains(connection)
    val requiredAliases: List[(Domain,Map[String,Boolean])] = Aliases.findRequiredAndCommonAliases(domains)
    val requiredRelays: Option[List[(Domain,Map[String,Boolean])]] = Relays.findRequiredAndCommonRelaysIfEnabled(connection,domains)
    Ok(views.html.alias.common( connection, requiredAliases, requiredRelays ))
  }

  def crossDomain(connection: ConnectionName) = ConnectionAction(connection) { implicit request =>
    val aliases = Aliases.customAliases
    val relayDomains = Domains.findDomains(connection)
    val customAliases: List[(Domain, (Map[String,Boolean], Option[Map[String,Boolean]]))] = relayDomains.map{ d =>
      ( d, d.findCustomAliasesAndRelays )
    }
    Ok(views.html.alias.cross(connection, aliases, customAliases) )
  }

  def disable(connection: ConnectionName, email: String) = ConnectionAction(connection).async { implicit connectionRequest =>
    AliasAction(email) { implicit aliasRequest => 
      aliasRequest.alias.disable(connection)
      Redirect(routes.AliasController.alias(connectionRequest.connection))      
    }(connectionRequest)
  }

  def enable(connection: ConnectionName, email: String) = ConnectionAction(connection).async { implicit connectionRequest =>
    AliasAction(email) { implicit aliasRequest => 
        aliasRequest.alias.enable(connection)
        Redirect(routes.AliasController.alias(connection))    
    }(connectionRequest)
  }

}

