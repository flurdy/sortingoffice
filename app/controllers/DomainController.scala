package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName

class RequestWithDomain[A](val domainRequested: Domain, request: Request[A]) extends WrappedRequest[A](request)

trait DomainInjector {

  def DomainAction(name: String) = new ActionBuilder[RequestWithDomain] {
    def invokeBlock[A](request: Request[A], block: (RequestWithDomain[A]) => Future[SimpleResult]) = {      
      request match { 
        case connectionRequest: RequestWithConnection[A] => {
          Domains.findDomain(connectionRequest.connection, name) match {
            case Some(domain) => {
              block(new RequestWithDomain(domain, connectionRequest))
            }
            case None => {
              Logger.warn(s"Domain $name not found")
              val relayDomains = Domains.findDomains(connectionRequest.connection)
              val backups = Domains.findBackupDomainsIfEnabled(connectionRequest.connection)
              implicit val errorMessages = List(ErrorMessage("Domain not found"))
              Future.successful(NotFound(views.html.domain.domain( 
                connectionRequest.connection, relayDomains, backups)(
                errorMessages,FeatureToggles.findFeatureToggles(connectionRequest.connection))))
            }
          }          
        }
        case _ => Future.successful(InternalServerError)
      }
    }
  }

  def BackupAction(name: String) = new ActionBuilder[RequestWithDomain] {
    def invokeBlock[A](request: Request[A], block: (RequestWithDomain[A]) => Future[SimpleResult]) = {      
      request match { 
        case connectionRequest: RequestWithConnection[A] => {
          Domains.findBackupDomain(connectionRequest.connection, name) match {
            case Some(domain) => {
              block(new RequestWithDomain(domain, connectionRequest))
            }
            case None => {
              Logger.warn(s"Backup domain $name not found")
              val relayDomains = Domains.findDomains(connectionRequest.connection)
              val backups = Domains.findBackupDomainsIfEnabled(connectionRequest.connection)
              implicit val errorMessages = List(ErrorMessage("Domain not found"))
              Future.successful(NotFound(views.html.domain.domain( 
                connectionRequest.connection, relayDomains, backups)(
                errorMessages,FeatureToggles.findFeatureToggles(connectionRequest.connection))))
            }
          }          
        }
        case _ => Future.successful(InternalServerError)
      }
    }
  }

}

object DomainController extends Controller with DbController with FeatureToggler with DomainInjector {

  def domain(connection: ConnectionName) = ConnectionAction(connection) { implicit request => 
    val relayDomains = Domains.findDomains(connection)
    val backups = Domains.findBackupDomainsIfEnabled(connection)
    Ok(views.html.domain.domain( connection, relayDomains, backups))
  }

  def alias(connection: ConnectionName, name: String) = ConnectionAction(connection) { implicit connectionRequest =>
    // DomainAction(connection, name) { implicit domainRequest => 
      Domains.findDomain(connection, name) match {
        case Some(domain) =>{
          val relays = domain.findRelaysIfEnabled
          val aliases = domain.findAliases
          val users = domain.findUsers
          Ok(views.html.domain.domainalias( connection, Some(domain), None, relays, aliases, users))
        }
        case None => {
          Domains.findBackupDomain(connection, name) match {
            case Some(domain) =>{
              val relays = domain.findRelaysIfEnabled
              val aliases = domain.findAliases
              val users = domain.findUsers
              Ok(views.html.domain.domainalias( connection, None, Some(domain), relays, aliases, users))
            }
            case None => {
              Logger.warn(s"Domain $name not found")
              val relayDomains = Domains.findDomains(connection)
              val backups = Domains.findBackupDomainsIfEnabled(connection)
              implicit val errorMessages = List(ErrorMessage("Domain not found"))
              NotFound(views.html.domain.domain( connection, relayDomains, backups))
            }
          }
        }
      }
    // }
  }

  def disable(connection: ConnectionName, name: String) = {
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(name) { implicit domainRequest =>
          domainRequest.domainRequested.disable
          Redirect(routes.DomainController.domain(connection))
      }(connectionRequest) 
    }
  }

  def enable(connection: ConnectionName, name: String) = {
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(name) { implicit domainRequest =>
          domainRequest.domainRequested.enable
          Redirect(routes.DomainController.domain(connection))
      }(connectionRequest) 
    }
  }

  def disableBackup(connection: ConnectionName, name: String) = { 
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(name) { implicit domainRequest =>
        domainRequest.domainRequested.disableBackup
        Redirect(routes.DomainController.domain(connection))
      }(connectionRequest) 
    }
  }

  def enableBackup(connection: ConnectionName, name: String) = { 
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(name) { implicit domainRequest =>
        domainRequest.domainRequested.enableBackup
        Redirect(routes.DomainController.domain(connection))
      }(connectionRequest) 
    }
  }

}
