package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName


object DomainController extends Controller with DbController {

  def domain(connection: ConnectionName) = ConnectionAction(connection) {
    val relayDomains = Domains.findDomains(connection)
    val backups = Domains.findBackupDomainsIfEnabled(connection)
    val toggle = FeatureToggles.isToggleEnabled(connection)
    Ok(views.html.domain.domain( connection, relayDomains, backups, toggle ))
  }

  def alias(connection: ConnectionName, name: String) = ConnectionAction(connection) {
    Domains.findDomain(connection, name) match {
      case Some(domain) =>{
        val relays = domain.findRelaysIfEnabled
        val aliases = domain.findAliases
        val users = domain.findUsers
        val toggle = FeatureToggles.isToggleEnabled(connection)
        Ok(views.html.domain.domainalias( connection, Some(domain), None, relays, aliases, users, toggle))
      }
      case None => {
        Domains.findBackupDomain(connection, name) match {
          case Some(domain) =>{
            val relays = domain.findRelaysIfEnabled
            val aliases = domain.findAliases
            val users = domain.findUsers
            val toggle = FeatureToggles.isToggleEnabled(connection)
            Ok(views.html.domain.domainalias( connection, None, Some(domain), relays, aliases, users, toggle))
          }
          case None => {
            Logger.warn(s"Domain $name not found")
            val relayDomains = Domains.findDomains(connection)
            val backups = Domains.findBackupDomainsIfEnabled(connection)
            val toggle = FeatureToggles.isToggleEnabled(connection)
            implicit val errorMessages = List(ErrorMessage("Domain not found"))
            NotFound(views.html.domain.domain( connection, relayDomains, backups, toggle))
          }
        }
      }
    }
  }

  def disable(connection: ConnectionName, name: String) = ConnectionAction(connection) {
    Domains.findDomain(connection, name) match {
      case Some(domain) =>{
        domain.disable
        Redirect(routes.DomainController.domain(connection))
      }
      case None => {
        Logger.warn(s"Domain $name not found")
        val relayDomains = Domains.findDomains(connection)
        val backups = Domains.findBackupDomainsIfEnabled(connection)
        val toggle = FeatureToggles.isToggleEnabled(connection)
        implicit val errorMessages = List(ErrorMessage("Domain not found"))
        NotFound(views.html.domain.domain( connection, relayDomains, backups, toggle))
      }
    }
  }

  def enable(connection: ConnectionName, name: String) = ConnectionAction(connection) {
    Domains.findDomain(connection, name) match {
      case Some(domain) =>{
        domain.enable
        Redirect(routes.DomainController.domain(connection))
      }
      case None => {
        Logger.warn(s"Domain $name not found")
        val relayDomains = Domains.findDomains(connection)
        val backups = Domains.findBackupDomainsIfEnabled(connection)
        val toggle = FeatureToggles.isToggleEnabled(connection)
        implicit val errorMessages = List(ErrorMessage("Domain not found"))
        NotFound(views.html.domain.domain( connection, relayDomains, backups, toggle))
      }
    }
  }

  def disableBackup(connection: ConnectionName, name: String) = ConnectionAction(connection) {
    Domains.findBackupDomain(connection, name) match {
      case Some(domain) =>{
        domain.disableBackup
        Redirect(routes.DomainController.domain(connection))
      }
      case None => {
        Logger.warn(s"Domain $name not found")
        val relayDomains = Domains.findDomains(connection)
        val backups = Domains.findBackupDomainsIfEnabled(connection)
        val toggle = FeatureToggles.isToggleEnabled(connection)
        implicit val errorMessages = List(ErrorMessage("Domain not found"))
        NotFound(views.html.domain.domain( connection, relayDomains, backups, toggle))
      }
    }
  }

  def enableBackup(connection: ConnectionName, name: String) = ConnectionAction(connection) {
    Domains.findBackupDomain(connection, name) match {
      case Some(domain) =>{
        domain.enableBackup
        Redirect(routes.DomainController.domain(connection))
      }
      case None => {
        Logger.warn(s"Domain $name not found")
        val relayDomains = Domains.findDomains(connection)
        val backups = Domains.findBackupDomainsIfEnabled(connection)
        val toggle = FeatureToggles.isToggleEnabled(connection)
        implicit val errorMessages = List(ErrorMessage("Domain not found"))
        NotFound(views.html.domain.domain( connection, relayDomains, backups, toggle))
      }
    }
  }


}
