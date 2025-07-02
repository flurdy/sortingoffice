package controllers

import javax.inject._
import scala.concurrent.{ExecutionContext, Future}
import play.api.mvc._
import play.api.Logging
import play.api.data._
import play.api.data.Forms._
import models._
import infrastructure.{UserRepository, DomainRepository, RelayRepository, AliasRepository}

@Singleton
class DomainController @Inject()(
  cc: MessagesControllerComponents,
  domains: Domains,
  featureToggles: FeatureToggles,
  environment: Environment,
  relays: Relays,
  aliases: Aliases,
  userRepository: UserRepository,
  domainRepository: DomainRepository,
  relayRepository: infrastructure.RelayRepository,
  aliasRepository: infrastructure.AliasRepository
)(implicit ec: ExecutionContext)
  extends AbstractController(cc)
  with Logging {

  val domainFormFields = single(
    "name" -> text
  )
  val domainForm = Form(domainFormFields)

  val backupFormFields = mapping(
    "connection" -> ignored(None: Option[String]),
    "name" -> text,
    "enabled" -> ignored(false),
    "transport" -> text
  )(Domain.apply)(Domain.unapply)
  val backupForm = Form(backupFormFields)

  val updateBackupFormFields = mapping(
    "connection" -> ignored(None: Option[String]),
    "name" -> ignored(""),
    "enabled" -> ignored(false),
    "transport" -> text
  )(Domain.apply)(Domain.unapply)
  val updateBackupForm = Form(updateBackupFormFields)

  // Helper: get feature toggles for a connection (for template rendering only)
  private def featureToggleMap(connection: String): FeatureToggleMap = featureToggles.findFeatureToggles(connection)

  def domain(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    val relayDomains = domains.findDomains(connection)
    val backups = domains.findBackupDomainsIfEnabled(connection, featureToggles)
    Ok(views.html.domain.domain(connection, relayDomains, backups))
  }

  def viewDomain(connection: String, name: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    val featureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, name) match {
      case Some(domain) =>
        val relaysForDomain = relayRepository.findRelaysForDomain(connection, domain)
        val aliasesForDomain = aliasRepository.findAllAliasesForDomain(domain)
        val usersForDomain = userRepository.findUsersForDomain(domain)
        val databaseDomains = domain.findInDatabases(environment, domains)
        Ok(views.html.domain.domaindetails(
          connection, Some(domain), None, Some(relaysForDomain), aliasesForDomain, usersForDomain, databaseDomains))
      case None =>
        val relayDomains = domains.findDomains(connection)
        val backups = domains.findBackupDomainsIfEnabled(connection, featureToggles)
        val errorMessages = List(ErrorMessage("Domain not found"))
        NotFound(views.html.domain.domain(
          connection, relayDomains, backups)(errorMessages, featureToggleMap, None))
    }
  }

  def viewBackup(connection: String, name: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    val featureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findBackupDomain(connection, name) match {
      case Some(backup) =>
        val relaysForBackup = relayRepository.findRelaysForDomain(connection, backup.domain)
        val aliasesForBackup = aliasRepository.findAllAliasesForDomain(backup.domain)
        val usersForBackup = userRepository.findUsersForDomain(backup.domain)
        val databaseDomains = backup.domain.findInDatabases(environment, domains)
        Ok(views.html.domain.domaindetails(
          connection, None, Some(backup), Some(relaysForBackup), aliasesForBackup, usersForBackup, databaseDomains))
      case None =>
        val relayDomains = domains.findDomains(connection)
        val backups = domains.findBackupDomainsIfEnabled(connection, featureToggles)
        val errorMessages = List(ErrorMessage("Backup domain not found"))
        NotFound(views.html.domain.domain(
          connection, relayDomains, backups)(errorMessages, featureToggleMap, None))
    }
  }

  def disable(connection: String, name: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    domains.findDomain(connection, name) match {
      case Some(domain) =>
        domain.disable(featureToggles, domainRepository)
        logger.info(s"Domain disabled: $name")
        val redirect = returnUrl match {
          case "domaindetails" => Redirect(routes.DomainController.viewDomain(connection, name))
          case _ => Redirect(routes.DomainController.domain(connection))
        }
        redirect
      case None =>
        val relayDomains = domains.findDomains(connection)
        val backups = domains.findBackupDomainsIfEnabled(connection, featureToggles)
        val errorMessages = List(ErrorMessage("Domain not found"))
        NotFound(views.html.domain.domain(
          connection, relayDomains, backups)(errorMessages, featureToggleMap(connection), None))
    }
  }

  def enable(connection: String, name: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    domains.findDomain(connection, name) match {
      case Some(domain) =>
        domain.enable(featureToggles, domainRepository)
        logger.info(s"Domain enabled: $name")
        val redirect = returnUrl match {
          case "domaindetails" => Redirect(routes.DomainController.viewDomain(connection, name))
          case _ => Redirect(routes.DomainController.domain(connection))
        }
        redirect
      case None =>
        val relayDomains = domains.findDomains(connection)
        val backups = domains.findBackupDomainsIfEnabled(connection, featureToggles)
        val errorMessages = List(ErrorMessage("Domain not found"))
        NotFound(views.html.domain.domain(
          connection, relayDomains, backups)(errorMessages, featureToggleMap(connection), None))
    }
  }

  def disableBackup(connection: String, name: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    domains.findBackupDomain(connection, name) match {
      case Some(backup) =>
        backup.disable(featureToggles, domainRepository)
        logger.info(s"Backup disabled: $name")
        val redirect = returnUrl match {
          case "domaindetails" => Redirect(routes.DomainController.viewDomain(connection, name))
          case _ => Redirect(routes.DomainController.domain(connection))
        }
        redirect
      case None =>
        val relayDomains = domains.findDomains(connection)
        val backups = domains.findBackupDomainsIfEnabled(connection, featureToggles)
        val errorMessages = List(ErrorMessage("Backup domain not found"))
        NotFound(views.html.domain.domain(
          connection, relayDomains, backups)(errorMessages, featureToggleMap(connection), None))
    }
  }

  def enableBackup(connection: String, name: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    domains.findBackupDomain(connection, name) match {
      case Some(backup) =>
        backup.enable(featureToggles, domainRepository)
        logger.info(s"Backup enabled: $name")
        val redirect = returnUrl match {
          case "domaindetails" => Redirect(routes.DomainController.viewDomain(connection, name))
          case _ => Redirect(routes.DomainController.domain(connection))
        }
        redirect
      case None =>
        val relayDomains = domains.findDomains(connection)
        val backups = domains.findBackupDomainsIfEnabled(connection, featureToggles)
        val errorMessages = List(ErrorMessage("Backup domain not found"))
        NotFound(views.html.domain.domain(
          connection, relayDomains, backups)(errorMessages, featureToggleMap(connection), None))
    }
  }

  def viewAdd(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    Ok(views.html.domain.addDomain(connection, domainForm))
  }

  def viewAddDatabaseDomain(connection: String, name: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    Ok(views.html.domain.addDomain(connection, domainForm.fill(name)))
  }

  def add(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    val featureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap
    implicit val currentUser: Option[ApplicationUser] = None
    domainForm.bindFromRequest().fold(
      errors => {
        logger.warn(s"Add domain form error")
        BadRequest(views.html.domain.addDomain(connection, errors))
      },
      nameDesired => {
        if (featureToggles.isAddEnabled(connection)) {
          domains.findDomain(connection, nameDesired) match {
            case None =>
              domains.findBackupDomain(connection, nameDesired) match {
                case None =>
                  domains.newDomain(connection, nameDesired).save(featureToggles, domainRepository)
                  logger.info(s"Domain added: $nameDesired")
                  Redirect(routes.DomainController.viewDomain(connection, nameDesired))
                case Some(_) =>
                  logger.warn(s"Domain backup $nameDesired already exists")
                  val errorMessages = List(ErrorMessage("Domain already exist"))
                  BadRequest(views.html.domain.addDomain(connection, domainForm.fill(nameDesired))(errorMessages, featureToggleMap, None))
              }
            case Some(_) =>
              logger.warn(s"Domain $nameDesired already exists")
              val errorMessages = List(ErrorMessage("Domain already exist"))
              BadRequest(views.html.domain.addDomain(connection, domainForm.fill(nameDesired))(errorMessages, featureToggleMap, None))
          }
        } else {
          logger.warn(s"Add feature not enabled")
          val errorMessages = List(ErrorMessage("Add feature not enabled"))
          BadRequest(views.html.domain.addDomain(connection, domainForm.fill(nameDesired))(errorMessages, featureToggleMap, None))
        }
      }
    )
  }

  def viewAddBackup(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    Ok(views.html.domain.addBackup(connection, backupForm))
  }

  def viewAddDatabaseBackup(connection: String, name: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    Ok(views.html.domain.addBackup(connection, backupForm.fill(new Domain(name))))
  }

  def addBackup(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    val featureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap
    implicit val currentUser: Option[ApplicationUser] = None
    backupForm.bindFromRequest().fold(
      errors => {
        logger.warn(s"Add backup domain form error")
        BadRequest(views.html.domain.addBackup(connection, errors))
      },
      backup => {
        domains.findDomain(connection, backup.name) match {
          case None =>
            domains.findBackupDomain(connection, backup.name) match {
              case None if featureToggles.isAddEnabled(connection) =>
                Backup(backup.withConnection(connection)).save(featureToggles, domainRepository)
                logger.info(s"Domain backup ${backup.name} added")
                Redirect(routes.DomainController.viewBackup(connection, backup.name))
              case None =>
                logger.warn(s"Add feature not enabled")
                val errorMessages = List(ErrorMessage("Add feature not enabled"))
                BadRequest(views.html.domain.addBackup(connection, backupForm.fill(backup))(errorMessages, featureToggleMap, None))
              case Some(_) =>
                logger.warn(s"Domain backup ${backup.name} already exists")
                val errorMessages = List(ErrorMessage("Backup domain already exist"))
                BadRequest(views.html.domain.addBackup(connection, backupForm.fill(backup))(errorMessages, featureToggleMap, None))
            }
          case Some(_) =>
            logger.warn(s"Domain ${backup.name} already exists")
            val errorMessages = List(ErrorMessage("Backup domain already exist"))
            BadRequest(views.html.domain.addBackup(connection, backupForm.fill(backup))(errorMessages, featureToggleMap, None))
        }
      }
    )
  }

  def viewRemove(connection: String, name: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    val featureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findDomain(connection, name) match {
      case Some(domain) =>
        val aliasesForDomain = aliasRepository.findAllAliasesForDomain(domain)
        val relaysForDomain = relayRepository.findRelaysForDomain(connection, domain)
        val usersForDomain = userRepository.findUsersForDomain(domain)
        Ok(views.html.domain.removeDomain(connection, domain, aliasesForDomain, relaysForDomain, usersForDomain))
      case None =>
        NotFound("Domain not found")
    }
  }

  def remove(connection: String, name: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    val featureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap
    implicit val currentUser: Option[ApplicationUser] = None
    if (featureToggles.isRemoveEnabled(connection)) {
      domains.findDomain(connection, name) match {
        case Some(domain) =>
          domain.delete(featureToggles, domainRepository)
          logger.info(s"Domain removed: $name")
          Redirect(routes.DomainController.domain(connection))
        case None =>
          domains.findBackupDomain(connection, name) match {
            case Some(backup) =>
              backup.delete(featureToggles, domainRepository)
              logger.info(s"Backup domain removed: $name")
              Redirect(routes.DomainController.domain(connection))
            case None =>
              logger.warn(s"Domain $name not found")
              val relayDomains = domains.findDomains(connection)
              val backups = domains.findBackupDomainsIfEnabled(connection, featureToggles)
              val errorMessages = List(ErrorMessage("Domain not found"))
              NotFound(views.html.domain.domain(connection, relayDomains, backups)(errorMessages, featureToggleMap, None))
          }
      }
    } else {
      logger.warn(s"Remove feature not enabled")
      val relayDomains = domains.findDomains(connection)
      val backups = domains.findBackupDomainsIfEnabled(connection, featureToggles)
      val errorMessages = List(ErrorMessage("Remove feature not enabled"))
      BadRequest(views.html.domain.domain(connection, relayDomains, backups)(errorMessages, featureToggleMap, None))
    }
  }

  def updateBackup(connection: String, name: String, returnUrl: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    val featureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = featureToggleMap
    implicit val currentUser: Option[ApplicationUser] = None
    domains.findBackupDomain(connection, name) match {
      case Some(backup) =>
        updateBackupForm.bindFromRequest().fold(
          errors => {
            logger.warn(s"Update backup domain form error")
            val relaysForBackup = relayRepository.findRelaysForDomain(connection, backup.domain)
            val aliasesForBackup = aliasRepository.findAllAliasesForDomain(backup.domain)
            val usersForBackup = userRepository.findUsersForDomain(backup.domain)
            val databaseDomains = backup.domain.findInDatabases(environment, domains)
            BadRequest(views.html.domain.domaindetails(connection, None, Some(backup), Some(relaysForBackup), aliasesForBackup, usersForBackup, databaseDomains))
          },
          backupData => {
            Backup(backup.domain.copy(transport = backupData.transport)).update(featureToggles, domainRepository)
            logger.info(s"Domain updated: $name")
            Redirect(routes.DomainController.viewDomain(connection, name))
          }
        )
      case None =>
        NotFound("Backup domain not found")
    }
  }

  def convertToBackup(connection: String, name: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    val featureToggleMap = featureToggles.findFeatureToggles(connection)
    domains.findDomain(connection, name) match {
      case Some(domain) =>
        domain.convertToBackup(featureToggles, domainRepository)
        Redirect(routes.DomainController.viewBackup(connection, name))
      case None =>
        NotFound("Domain not found")
    }
  }

  def convertToRelay(connection: String, name: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    val featureToggleMap = featureToggles.findFeatureToggles(connection)
    domains.findBackupDomain(connection, name) match {
      case Some(backup) =>
        backup.convertToRelay(featureToggles, domainRepository)
        Redirect(routes.DomainController.viewDomain(connection, name))
      case None =>
        NotFound("Backup domain not found")
    }
  }
}
