package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import play.api.data._
import play.api.data.Forms._
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

object AliasController extends Controller with DbController with FeatureToggler with AliasInjector with DomainInjector {

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

  val aliasFormFields = mapping (
    "mail" -> text,
    "destination" -> text,
    "enabled" -> ignored(false)
  )(Alias.apply)(Alias.unapply)

  val aliasForm = Form( aliasFormFields )

  def viewAdd(connection: ConnectionName, domainName: String) = ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName) { implicit domainRequest =>
        Ok(views.html.alias.addAlias( connection, domainRequest.domainRequested, aliasForm ))
      }(connectionRequest)
  }

  def add(connection: ConnectionName, domainName: String) = ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName) { domainRequest =>
        aliasForm.bindFromRequest.fold(
          errors => {
            Logger.warn(s"Add alias form error")
            BadRequest(views.html.alias.addAlias( connection, domainRequest.domainRequested, errors ))
          },
          aliasToAdd => {
            Aliases.findAlias(connectionRequest.connection, aliasToAdd.mail) match {
              case None if FeatureToggles.isAddEnabled(connectionRequest.connection) => {
                aliasToAdd.save(connection)
                Redirect(routes.DomainController.alias(connection,domainName))
              }
              case None => {
                Logger.warn(s"Add feature not enabled")
                implicit val errorMessages = List(ErrorMessage("Add feature not enabled"))
                BadRequest(views.html.alias.addAlias( connection, domainRequest.domainRequested, aliasForm.fill(aliasToAdd)))
              }
              case Some(_) => {
                Logger.warn(s"Alias ${aliasToAdd.mail} already exists")
                implicit val errorMessages = List(ErrorMessage("Alias already exist"))
                BadRequest(views.html.alias.addAlias( connection, domainRequest.domainRequested, aliasForm.fill(aliasToAdd)))
              }
            }
          }
        )
      }(connectionRequest)
  }

}
