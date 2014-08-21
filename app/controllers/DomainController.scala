package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName


object DomainController extends Controller with DbController with FeatureToggler {

  def domain(connection: ConnectionName) = ConnectionAction(connection) { implicit request =>
    val relayDomains = Domains.findDomains(connection)
    val backups = Domains.findBackupDomainsIfEnabled(connection)
    Ok(views.html.domain.domain( connection, relayDomains, backups))
  }

  def alias(connection: ConnectionName, name: String) = ConnectionAction(connection) { implicit request =>
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
  }

  def disable(connection: ConnectionName, name: String) = ConnectionAction(connection) { implicit request =>
    Domains.findDomain(connection, name) match {
      case Some(domain) =>{
        domain.disable
        Redirect(routes.DomainController.domain(connection))
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

  def enable(connection: ConnectionName, name: String) = ConnectionAction(connection) { implicit request =>
    Domains.findDomain(connection, name) match {
      case Some(domain) =>{
        domain.enable
        Redirect(routes.DomainController.domain(connection))
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

  def disableBackup(connection: ConnectionName, name: String) = ConnectionAction(connection) { implicit request =>
    Domains.findBackupDomain(connection, name) match {
      case Some(domain) =>{
        domain.disableBackup
        Redirect(routes.DomainController.domain(connection))
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

  def enableBackup(connection: ConnectionName, name: String) = ConnectionAction(connection) { implicit request =>
    Domains.findBackupDomain(connection, name) match {
      case Some(domain) =>{
        domain.enableBackup
        Redirect(routes.DomainController.domain(connection))
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
