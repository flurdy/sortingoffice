package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName


object DomainController extends Controller with DbController {

  def domain(connection: ConnectionName) = ConnectionAction(connection) {
    val relayDomains = Domains.findRelayDomains(connection)
    val backups = Domains.findBackupDomainsIfEnabled(connection)
    Ok(views.html.domain.domain( connection, relayDomains, backups ))
  }

  def alias(connection: ConnectionName, name: String) = ConnectionAction(connection) {
    Domains.findRelayDomain(connection, name) match {
      case Some(domain) =>{
        val relays = domain.findRelaysIfEnabled
        val aliases = domain.findAliases
        val users = domain.findUsers
        Ok(views.html.domain.domainalias( connection, domain,relays,aliases,users))
      }
      case None => {
        Logger.warn(s"Domain $name not found")
        val relayDomains = Domains.findRelayDomains(connection)
        val backups = Domains.findBackupDomainsIfEnabled(connection)
        implicit val errorMessages = List(ErrorMessage("Domain not found"))
        NotFound(views.html.domain.domain( connection, relayDomains, backups))
      }
    }
  }

}

