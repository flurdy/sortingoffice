package controllers

import javax.inject._
import scala.concurrent.{ExecutionContext, Future}
import play.api.mvc._
import play.api.Logging
import play.api.data._
import play.api.data.Forms._
import models._
import infrastructure._

@Singleton
class AliasController @Inject()(
  cc: MessagesControllerComponents,
  aliases: Aliases,
  featureToggles: FeatureToggles,
  domains: Domains,
  relays: Relays,
  users: Users,
  aliasRepository: AliasRepository
)(implicit ec: ExecutionContext)
  extends AbstractController(cc)
  with Logging {

  val aliasFormFields = mapping(
    "mail" -> text,
    "destination" -> text,
    "enabled" -> ignored(false)
  )(Alias.apply)(Alias.unapply)
  val aliasForm = Form(aliasFormFields)

  val updateAliasFormFields = mapping(
    "mail" -> ignored(""),
    "destination" -> text,
    "enabled" -> ignored(false)
  )(Alias.apply)(Alias.unapply)
  val updateAliasForm = Form(updateAliasFormFields)

  // Helper: get feature toggles for a connection (for template rendering only)
  private def featureToggleMap(connection: String): FeatureToggleMap = featureToggles.findFeatureToggles(connection)

  def alias(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    Ok(views.html.alias.alias(connection))
  }

  def catchAll(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    val allDomains = domains.findDomains(connection)
    val catchAllAliases = aliases.findCatchAllDomains(connection, allDomains)
    val allBackups = domains.findBackupDomains(connection).map(_.domain)
    val catchAllRelays = relays.findCatchAllDomainsIfEnabled(connection, allDomains)
    val catchAllBackupRelays = relays.findCatchAllBackupsIfEnabled(connection, allBackups)
    Ok(views.html.alias.catchall(
      connection,
      catchAllAliases._1,
      catchAllAliases._2,
      catchAllRelays.map(_._1),
      catchAllBackupRelays.map(_._1),
      catchAllRelays.map(_._2),
      catchAllBackupRelays.map(_._2)
    ))
  }

  def common(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    val ds = domains.findDomains(connection)
    val requiredAliases = aliases.findRequiredAndCommonAliases(ds)
    val requiredRelays = relays.findRequiredAndCommonRelaysIfEnabled(connection, ds)
    Ok(views.html.alias.common(connection, requiredAliases, requiredRelays))
  }

  def crossDomain(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    val customAliases = aliases.customAliases
    val relayDomains = domains.findDomains(connection)
    val customAliasesMap = relayDomains.map { d =>
      (d, d.findCustomAliasesAndRelays(aliases, relays, featureToggles))
    }
    Ok(views.html.alias.cross(connection, customAliases, customAliasesMap))
  }

  def orphan(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    val ds = domains.findDomains(connection)
    val backups = domains.findBackupDomains(connection).map(_.domain)
    val orphanAliases = aliases.findOrphanAliases(connection, ds)
    val orphanRelays = relays.findOrphanRelays(connection, ds ++ backups)
    val orphanUsers = users.findOrphanUsers(connection, ds)
    Ok(views.html.alias.orphan(connection, orphanAliases, orphanRelays, orphanUsers))
  }

  def all(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    val allAliases = aliases.findAllAliases(connection)
    Ok(views.html.alias.all(connection, allAliases))
  }

  def disable(connection: String, domainName: String, email: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    aliases.findAlias(connection, email) match {
      case Some(alias) =>
        alias.disable(connection, featureToggles, aliasRepository)
        logger.info(s"Alias $email disabled")
        returnUrl match {
          case "catchall" => Redirect(routes.AliasController.catchAll(connection))
          case "aliasdetails" => Redirect(routes.AliasController.viewAlias(connection, domainName, email))
          case "userdetails" => Redirect(routes.UserController.viewUser(connection, email))
          case "removedomain" => Redirect(routes.DomainController.viewRemove(connection, domainName))
          case "allalias" => Redirect(routes.AliasController.all(connection))
          case _ => Redirect(routes.DomainController.viewDomain(connection, domainName))
        }
      case None => NotFound("Alias not found")
    }
  }

  def enable(connection: String, domainName: String, email: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    aliases.findAlias(connection, email) match {
      case Some(alias) =>
        alias.enable(connection, featureToggles, aliasRepository)
        logger.info(s"Alias $email enabled")
        returnUrl match {
          case "catchall" => Redirect(routes.AliasController.catchAll(connection))
          case "aliasdetails" => Redirect(routes.AliasController.viewAlias(connection, domainName, email))
          case "userdetails" => Redirect(routes.UserController.viewUser(connection, email))
          case "allalias" => Redirect(routes.AliasController.all(connection))
          case _ => Redirect(routes.DomainController.viewDomain(connection, domainName))
        }
      case None => NotFound("Alias not found")
    }
  }

  def viewAdd(connection: String, domainName: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, domainName) match {
      case Some(domain) => Ok(views.html.alias.addAlias(connection, domain, aliasForm, "domaindetails"))
      case None => NotFound("Domain not found")
    }
  }

  def viewAddAlias(connection: String, domainName: String, mail: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, domainName) match {
      case Some(domain) => Ok(views.html.alias.addAlias(connection, domain, aliasForm.fill(Alias(mail, "", false)), "aliasdetails"))
      case None => NotFound("Domain not found")
    }
  }

  def viewAddCatchAll(connection: String, domainName: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, domainName) match {
      case Some(domain) => Ok(views.html.alias.addAlias(connection, domain, aliasForm.fill(Alias(s"@$domainName", "", false)), "catchall"))
      case None => NotFound("Domain not found")
    }
  }

  def add(connection: String, domainName: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    aliasForm.bindFromRequest().fold(
      errors => {
        logger.warn(s"Add alias form error")
        implicit val errorMessages: List[ErrorMessage] = List.empty
        implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
        implicit val currentUser: Option[ApplicationUser] = None
        domains.findDomain(connection, domainName) match {
          case Some(domain) => BadRequest(views.html.alias.addAlias(connection, domain, errors, returnUrl))
          case None => NotFound("Domain not found")
        }
      },
      aliasToAdd => {
        aliases.findAlias(connection, aliasToAdd.mail) match {
          case None if featureToggles.isAddEnabled(connection) =>
            aliasToAdd.save(connection, featureToggles, aliasRepository)
            logger.info(s"Alias ${aliasToAdd.mail} added")
            returnUrl match {
              case "catchall" => Redirect(routes.AliasController.catchAll(connection))
              case _ => Redirect(routes.DomainController.viewDomain(connection, domainName))
            }
          case None =>
            logger.warn(s"Add feature not enabled")
            implicit val errorMessages: List[ErrorMessage] = List(ErrorMessage("Add feature not enabled"))
            implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
            implicit val currentUser: Option[ApplicationUser] = None
            domains.findDomain(connection, domainName) match {
              case Some(domain) =>
                BadRequest(views.html.alias.addAlias(connection, domain, aliasForm.fill(aliasToAdd), returnUrl))
              case None => NotFound("Domain not found")
            }
          case Some(_) =>
            logger.warn(s"Alias ${aliasToAdd.mail} already exists")
            implicit val errorMessages: List[ErrorMessage] = List(ErrorMessage("Alias already exist"))
            implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
            implicit val currentUser: Option[ApplicationUser] = None
            domains.findDomain(connection, domainName) match {
              case Some(domain) =>
                BadRequest(views.html.alias.addAlias(connection, domain, aliasForm.fill(aliasToAdd), returnUrl))
              case None => NotFound("Domain not found")
            }
        }
      }
    )
  }

  def remove(connection: String, domainName: String, email: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    aliases.findAlias(connection, email) match {
      case Some(alias) =>
        alias.delete(connection, featureToggles, aliasRepository)
        logger.info(s"Alias $email deleted")
        returnUrl match {
          case "catchall" => Redirect(routes.AliasController.catchAll(connection))
          case "removedomain" => Redirect(routes.DomainController.viewRemove(connection, domainName))
          case _ => Redirect(routes.DomainController.viewDomain(connection, domainName))
        }
      case None => NotFound("Alias not found")
    }
  }

  def viewAlias(connection: String, domainName: String, email: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggles: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, domainName) match {
      case Some(domain) =>
        aliases.findAlias(connection, email) match {
          case Some(alias) =>
            val relaysForAlias = relays.findRelaysForAliasIfEnabled(connection, domain, alias)
            val databaseAliases = alias.findInDatabases
            Ok(views.html.alias.aliasdetails(connection, domain, alias, relaysForAlias, databaseAliases))
          case None => NotFound("Alias not found")
        }
      case None => NotFound("Domain not found")
    }
  }

  def updateAlias(connection: String, domainName: String, email: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, domainName) match {
      case Some(domain) =>
        aliases.findAlias(connection, email) match {
          case Some(alias) =>
            updateAliasForm.bindFromRequest().fold(
              errors => {
                val relaysForAlias = relays.findRelaysForAliasIfEnabled(connection, domain, alias)
                val databaseAliases = alias.findInDatabases
                BadRequest(views.html.alias.aliasdetails(connection, domain, alias, relaysForAlias, databaseAliases))
              },
              aliasData => {
                alias.copy(destination = aliasData.destination).update(connection, featureToggles, aliasRepository)
                logger.info(s"Alias $email updated")
                Redirect(routes.AliasController.viewAlias(connection, domainName, email))
              }
            )
          case None => NotFound("Alias not found")
        }
      case None => NotFound("Domain not found")
    }
  }
}
