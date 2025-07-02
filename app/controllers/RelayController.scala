package controllers

import javax.inject._
import scala.concurrent.{ExecutionContext, Future}
import play.api.mvc._
import play.api.Logging
import play.api.data._
import play.api.data.Forms._
import models._
import infrastructure.RelayRepository

@Singleton
class RelayController @Inject()(
  cc: MessagesControllerComponents,
  relays: Relays,
  featureToggles: FeatureToggles,
  domains: Domains,
  environment: Environment,
  relayRepository: RelayRepository
)(implicit ec: ExecutionContext)
  extends AbstractController(cc)
  with Logging {

  val relayFormFields = mapping(
    "recipient" -> text,
    "enabled" -> ignored(false),
    "status" -> text
  )(Relay.apply)(Relay.unapply)
  val relayForm = Form(relayFormFields)

  // Helper: get feature toggles for a connection (for template rendering only)
  private def togglesMap(connection: String): FeatureToggleMap = featureToggles.findFeatureToggles(connection)

  def disable(connection: String, domainName: String, recipient: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    relays.findRelay(connection, recipient) match {
      case Some(relay) =>
        relay.disable(connection, featureToggles, relayRepository)
        logger.info(s"Relay disabled: $recipient")
        val redirect = returnUrl match {
          case "catchall" => Redirect(routes.AliasController.catchAll(connection))
          case "removedomain" => Redirect(routes.DomainController.viewRemove(connection, domainName))
          case "relaydetails" => Redirect(routes.RelayController.viewRelay(connection, domainName, recipient))
          case _ => Redirect(routes.DomainController.viewDomain(connection, domainName))
        }
        redirect
      case None =>
        NotFound("Relay not found")
    }
  }

  def enable(connection: String, domainName: String, recipient: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    relays.findRelay(connection, recipient) match {
      case Some(relay) =>
        relay.enable(connection, featureToggles, relayRepository)
        logger.info(s"Relay enabled: $recipient")
        val redirect = returnUrl match {
          case "catchall" => Redirect(routes.AliasController.catchAll(connection))
          case "relaydetails" => Redirect(routes.RelayController.viewRelay(connection, domainName, recipient))
          case _ => Redirect(routes.DomainController.viewDomain(connection, domainName))
        }
        redirect
      case None =>
        NotFound("Relay not found")
    }
  }

  def viewAdd(connection: String, domainName: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggleMap: FeatureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, domainName) match {
      case Some(domain) =>
        Ok(views.html.relay.addRelay(connection, domain, relayForm, "domaindetails"))
      case None =>
        NotFound("Domain not found")
    }
  }

  def viewAddRelay(connection: String, domainName: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggleMap: FeatureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, domainName) match {
      case Some(domain) =>
        Ok(views.html.relay.addRelay(connection, domain, relayForm.fill(Relay(recipient, false, "OK")), "relaydetails"))
      case None =>
        NotFound("Domain not found")
    }
  }

  def viewAddCatchAll(connection: String, domainName: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggleMap: FeatureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, domainName) match {
      case Some(domain) =>
        Ok(views.html.relay.addRelay(connection, domain, relayForm.fill(Relay(s"@$domainName", false, "OK")), "catchall"))
      case None =>
        NotFound("Domain not found")
    }
  }

  def add(connection: String, domainName: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    val togglesMap = featureToggles.findFeatureToggles(connection)
    implicit val featureTogglesMap: FeatureToggleMap = togglesMap
    implicit val currentUser: Option[ApplicationUser] = None
    relayForm.bindFromRequest().fold(
      errors => {
        logger.warn(s"Add relay form error")
        domains.findDomain(connection, domainName) match {
          case Some(domain) => BadRequest(views.html.relay.addRelay(connection, domain, errors, returnUrl))
          case None => NotFound("Domain not found")
        }
      },
      relay => {
        relays.findRelay(connection, relay.recipient) match {
          case None if featureToggles.isAddEnabled(connection) =>
            relay.save(connection, featureToggles, relayRepository)
            val redirect = returnUrl match {
              case "catchall" => Redirect(routes.AliasController.catchAll(connection))
              case "aliasdetails" => Redirect(routes.AliasController.catchAll(connection))
              case _ => Redirect(routes.DomainController.viewDomain(connection, domainName))
            }
            redirect
          case None =>
            logger.warn(s"Add feature not enabled")
            domains.findDomain(connection, domainName) match {
              case Some(domain) =>
                val errorMessages = List(ErrorMessage("Add feature not enabled"))
                BadRequest(views.html.relay.addRelay(connection, domain, relayForm.fill(relay), returnUrl)(errorMessages, togglesMap, None))
              case None => NotFound("Domain not found")
            }
          case Some(_) =>
            logger.warn(s"Relay ${relay.recipient} already exists")
            domains.findDomain(connection, domainName) match {
              case Some(domain) =>
                val errorMessages = List(ErrorMessage("Relay already exist"))
                BadRequest(views.html.relay.addRelay(connection, domain, relayForm.fill(relay), returnUrl)(errorMessages, togglesMap, None))
              case None => NotFound("Domain not found")
            }
        }
      }
    )
  }

  def remove(connection: String, domainName: String, recipient: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    relays.findRelay(connection, recipient) match {
      case Some(relay) =>
        relay.delete(connection, featureToggles, relayRepository)
        logger.info(s"Relay $recipient removed")
        val redirect = returnUrl match {
          case "catchall" => Redirect(routes.AliasController.catchAll(connection))
          case "removedomain" => Redirect(routes.DomainController.viewRemove(connection, domainName))
          case _ => Redirect(routes.DomainController.viewDomain(connection, domainName))
        }
        redirect
      case None =>
        NotFound("Relay not found")
    }
  }

  def viewRelay(connection: String, domainName: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggleMap: FeatureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, domainName) match {
      case Some(domain) =>
        relays.findRelay(connection, recipient) match {
          case Some(relay) =>
            val backup = domains.findBackupDomain(connection, domainName)
            val databaseRelays = relay.findInDatabases(environment, relays)
            Ok(views.html.relay.relaydetails(connection, domain, backup, None, relay, databaseRelays))
          case None => NotFound("Relay not found")
        }
      case None => NotFound("Domain not found")
    }
  }

  def rejectRelay(connection: String, domainName: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    relays.findRelay(connection, recipient) match {
      case Some(relay) =>
        relay.reject(connection, featureToggles, relayRepository)
        logger.info(s"Relay $recipient set to reject")
        Redirect(routes.RelayController.viewRelay(connection, domainName, recipient))
      case None => NotFound("Relay not found")
    }
  }

  def acceptRelay(connection: String, domainName: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    relays.findRelay(connection, recipient) match {
      case Some(relay) =>
        relay.accept(connection, featureToggles, relayRepository)
        logger.info(s"Relay $recipient set to accept")
        Redirect(routes.RelayController.viewRelay(connection, domainName, recipient))
      case None => NotFound("Relay not found")
    }
  }

  def disableRelay(connection: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    NotImplemented
  }

  def enableAliasRelay(connection: String, domain: String, email: String, recipient: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    NotImplemented
  }

  def disableAliasRelay(connection: String, domain: String, email: String, recipient: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    NotImplemented
  }

  def addAliasRelay(connection: String, domain: String, email: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    NotImplemented
  }

  def removeRelay(connection: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    NotImplemented
  }

  def removeAliasRelay(connection: String, domain: String, email: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    NotImplemented
  }

  def viewAliasRelay(connection: String, domain: String, email: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    NotImplemented
  }

  def rejectAliasRelay(connection: String, domain: String, email: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    NotImplemented
  }

  def acceptAliasRelay(connection: String, domain: String, email: String, recipient: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    NotImplemented
  }
}
