package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import play.api.data._
import play.api.data.Forms._
import models._
import models.Environment.ConnectionName

class RequestWithDomain[A](val domainRequested: Domain, request: Request[A]) extends WrappedRequest[A](request)

class RequestWithBackup[A](val backup: Backup, request: Request[A]) extends WrappedRequest[A](request)

trait DomainInjector {

  def DomainAction(name: String) = new ActionBuilder[RequestWithDomain] {
    def invokeBlock[A](request: Request[A], block: (RequestWithDomain[A]) => Future[SimpleResult]) = {
      request match {
        case connectionRequest: RequestWithConnection[A] => {
          Domains.findDomain(connectionRequest.connection, name) match {
            case Some(domain) => {
              Logger.debug(s"Domain $name found")
              block(new RequestWithDomain(domain, connectionRequest))
            }
            case None => {
              Logger.warn(s"Domain $name not found")
              val relayDomains = Domains.findDomains(connectionRequest.connection)
              val backups = Domains.findBackupDomainsIfEnabled(connectionRequest.connection)
              implicit val errorMessages = List(ErrorMessage("Domain not found"))
              Future.successful(NotFound(views.html.domain.domain(
                connectionRequest.connection, relayDomains, backups)(
                errorMessages,FeatureToggles.findFeatureToggles(connectionRequest.connection),None)))
            }
          }
        }
        case _ => Future.successful(InternalServerError)
      }
    }
  }

  def BackupAction(name: String) = new ActionBuilder[RequestWithBackup] {
    def invokeBlock[A](request: Request[A], block: (RequestWithBackup[A]) => Future[SimpleResult]) = {
      request match {
        case connectionRequest: RequestWithConnection[A] => {
          Domains.findBackupDomain(connectionRequest.connection, name) match {
            case Some(backup) => {
              block(new RequestWithBackup(backup, connectionRequest))
            }
            case None => {
              Logger.warn(s"Backup domain $name not found")
              val relayDomains = Domains.findDomains(connectionRequest.connection)
              val backups = Domains.findBackupDomainsIfEnabled(connectionRequest.connection)
              implicit val errorMessages = List(ErrorMessage("Domain not found"))
              Future.successful(NotFound(views.html.domain.domain(
                connectionRequest.connection, relayDomains, backups)(
                errorMessages,FeatureToggles.findFeatureToggles(connectionRequest.connection),None)))
            }
          }
        }
        case _ => Future.successful(InternalServerError)
      }
    }
  }

  def DomainOrBackupAction(name: String) = new ActionBuilder[RequestWithDomain] {
    def invokeBlock[A](request: Request[A], block: (RequestWithDomain[A]) => Future[SimpleResult]) = {
      request match {
        case connectionRequest: RequestWithConnection[A] => {
          Domains.findDomain(connectionRequest.connection, name) match {
            case Some(domain) => {
              block(new RequestWithDomain(domain, connectionRequest))
            }
            case None => {
              Domains.findBackupDomain(connectionRequest.connection, name) match {
                case Some(backup) => {
                  block(new RequestWithDomain(backup.domain, connectionRequest))
                }
                case None => {
                  Logger.warn(s"Domain $name not found")
                  val relayDomains = Domains.findDomains(connectionRequest.connection)
                  val backups = Domains.findBackupDomainsIfEnabled(connectionRequest.connection)
                  implicit val errorMessages = List(ErrorMessage("Domain not found"))
                  Future.successful(NotFound(views.html.domain.domain(
                    connectionRequest.connection, relayDomains, backups)(
                    errorMessages,FeatureToggles.findFeatureToggles(connectionRequest.connection),None)))
                }
              }
            }
          }
        }
        case _ => Future.successful(InternalServerError)
      }
    }
  }

}

object DomainController extends Controller with DbController with FeatureToggler with DomainInjector with Secured {

  def domain(connection: ConnectionName) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection) { implicit connectionRequest =>
      val relayDomains = Domains.findDomains(connection)
      val backups = Domains.findBackupDomainsIfEnabled(connection)
      Ok(views.html.domain.domain( connection, relayDomains, backups))
    }(authRequest)
  }

  def viewDomain(connection: ConnectionName, name: String) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(name) { implicit domainRequest =>
        val relays  = domainRequest.domainRequested.findRelaysIfEnabled
        val aliases = domainRequest.domainRequested.findAliases
        val users   = domainRequest.domainRequested.findUsers
        val databaseDomains = domainRequest.domainRequested.findInDatabases
        Ok(views.html.domain.domaindetails(
          connection, Some(domainRequest.domainRequested), None, relays, aliases, users, databaseDomains))
      }(connectionRequest)
    }(authRequest)
  }

  def viewBackup(connection: ConnectionName, name: String) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      BackupAction(name) { implicit domainRequest =>
        val relays  = domainRequest.backup.domain.findRelaysIfEnabled
        val aliases = domainRequest.backup.domain.findAliases
        val users   = domainRequest.backup.domain.findUsers
        val databaseDomains = domainRequest.backup.domain.findInDatabases
        Ok(views.html.domain.domaindetails(
          connection, None, Some(domainRequest.backup), relays, aliases, users, databaseDomains))
      }(connectionRequest)
    }(authRequest)
  }

  def disable(connection: ConnectionName, name: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(name) { implicit domainRequest =>
        domainRequest.domainRequested.disable
        Logger.info(s"Domain disabled: $name")
        returnUrl match {
          case "domaindetails" => Redirect(routes.DomainController.viewDomain(connection,name))
          case _ => Redirect(routes.DomainController.domain(connection))
        }
      }(connectionRequest)
    }(authRequest)
  }

  def enable(connection: ConnectionName, name: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(name) { implicit domainRequest =>
        domainRequest.domainRequested.enable
        Logger.info(s"Domain enabled: $name")
        returnUrl match {
          case "domaindetails" => Redirect(routes.DomainController.viewDomain(connection,name))
          case _ => Redirect(routes.DomainController.domain(connection))
        }
      }(connectionRequest)
    }(authRequest)
  }

  def disableBackup(connection: ConnectionName, name: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      BackupAction(name) { implicit domainRequest =>
        domainRequest.backup.disable
        Logger.info(s"Backup disabled: $name")
        returnUrl match {
          case "domaindetails" => Redirect(routes.DomainController.viewDomain(connection,name))
          case _ => Redirect(routes.DomainController.domain(connection))
        }
      }(connectionRequest)
    }(authRequest)
  }

  def enableBackup(connection: ConnectionName, name: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      BackupAction(name) { implicit domainRequest =>
        domainRequest.backup.enable
        Logger.info(s"Backup enabled: $name")
        returnUrl match {
          case "domaindetails" => Redirect(routes.DomainController.viewDomain(connection,name))
          case _ => Redirect(routes.DomainController.domain(connection))
        }
      }(connectionRequest)
    }(authRequest)
  }

  val domainFormFields = single(
    "name" -> text
  )

  val domainForm = Form( domainFormFields )

  def viewAdd(connection: ConnectionName) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      Ok(views.html.domain.addDomain( connection, domainForm ))
    }(authRequest)
  }

  def add(connection: ConnectionName) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      domainForm.bindFromRequest()(request).fold(
        errors => {
          Logger.warn(s"Add domain form error")
          BadRequest(views.html.domain.addDomain(connection,errors))
        },
        nameDesired => {
          if(FeatureToggles.isAddEnabled(request.connection)){
            Domains.findDomain(connection, nameDesired) match {
              case None => {
                Domains.findBackupDomain(connection, nameDesired) match {
                  case None => {
                    Domains.newDomain(connection,nameDesired).save
                    Logger.info(s"Domain added: $nameDesired")
                    Redirect(routes.DomainController.viewDomain(connection,nameDesired))
                  }
                  case Some(_) => {
                    Logger.warn(s"Domain backup $nameDesired already exists")
                   implicit val errorMessages = List(ErrorMessage("Domain already exist"))
                    BadRequest(views.html.domain.addDomain(connection,domainForm.fill(nameDesired)))
                  }
                }
              }
              case Some(_) => {
                Logger.warn(s"Domain $nameDesired already exists")
                implicit val errorMessages = List(ErrorMessage("Domain already exist"))
                BadRequest(views.html.domain.addDomain(connection,domainForm.fill(nameDesired)))
              }
            }
          } else {
            Logger.warn(s"Add feature not enabled")
            implicit val errorMessages = List(ErrorMessage("Add feature not enabled"))
            BadRequest(views.html.domain.addDomain(connection,domainForm.fill(nameDesired)))
          }
        }
      )
    }(authRequest)
  }


  val backupFormFields = mapping(
    "connection" -> ignored(None:Option[ConnectionName]),
    "name" -> text,
    "enabled" -> ignored(false),
    "transport" -> text
  )(Domain.apply)(Domain.unapply)

  val backupForm = Form( backupFormFields )

  def viewAddBackup(connection: ConnectionName) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      Ok(views.html.domain.addBackup( connection, backupForm ))
    }(authRequest)
  }

  def addBackup(connection: ConnectionName) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      backupForm.bindFromRequest()(request).fold(
        errors => {
          Logger.warn(s"Add backup domain form error")
          BadRequest(views.html.domain.addBackup( connection, errors))
        },
        backup => {
          Domains.findDomain(connection, backup.name) match {
            case None => {
              Domains.findBackupDomain(connection, backup.name) match {
                case None if FeatureToggles.isAddEnabled(request.connection) => {
                  Backup(backup.withConnection(request.connection)).save
                  Logger.info(s"Domain backup ${backup.name} added")
                  Redirect(routes.DomainController.viewBackup(connection,backup.name))
                }
                case None  => {
                  Logger.warn(s"Add feature not enabled")
                  implicit val errorMessages = List(ErrorMessage("Add feature not enabled"))
                  BadRequest(views.html.domain.addBackup( connection, backupForm.fill(backup)))
                }
                case Some(_) => {
                  Logger.warn(s"Domain backup ${backup.name} already exists")
                  implicit val errorMessages = List(ErrorMessage("Backup domain already exist"))
                  BadRequest(views.html.domain.addBackup( connection, backupForm.fill(backup)))
                }
              }
            }
            case Some(_) => {
              Logger.warn(s"Domain ${backup.name} already exists")
              implicit val errorMessages = List(ErrorMessage("Backup domain already exist"))
              BadRequest(views.html.domain.addBackup( connection, backupForm.fill(backup)))
            }
          }
        }
      )
    }(authRequest)
  }

  def viewRemove(connection: ConnectionName, name: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(name) { implicit domainRequest =>
        val aliases = domainRequest.domainRequested.findAliases
        val relays  = domainRequest.domainRequested.findRelaysIfEnabled.getOrElse(Nil)
        val users   = domainRequest.domainRequested.findUsers
        Ok(views.html.domain.removeDomain( connection, domainRequest.domainRequested, aliases, relays, users ))
      }(connectionRequest)
    }(authRequest)
  }

  def remove(connection: ConnectionName, name: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection) { implicit connectionRequest =>
      if(FeatureToggles.isRemoveEnabled(connectionRequest.connection) ){
        Domains.findDomain(connection, name) match {
          case Some(domain) => {
            domain.delete
            Logger.info(s"Domain removed: $name")
            Redirect(routes.DomainController.domain(connection))
          }
          case None => {
            Domains.findBackupDomain(connection, name) match {
              case Some(backup) => {
                backup.delete
                Logger.info(s"Backup domain removed: $name")
                Redirect(routes.DomainController.domain(connection))
              }
              case None  => {
                Logger.warn(s"Domain $name not found")
                val relayDomains = Domains.findDomains(connectionRequest.connection)
                val backups = Domains.findBackupDomainsIfEnabled(connectionRequest.connection)
                implicit val errorMessages = List(ErrorMessage("Domain not found"))
                NotFound(views.html.domain.domain(connectionRequest.connection, relayDomains, backups))
              }
            }
          }
        }
      } else {
        Logger.warn(s"Remove feature not enabled")
        val relayDomains = Domains.findDomains(connection)
        val backups = Domains.findBackupDomainsIfEnabled(connection)
        implicit val errorMessages = List(ErrorMessage("Remove feature not enabled"))
        BadRequest(views.html.domain.domain(connectionRequest.connection, relayDomains, backups))
      }
    }(authRequest)
  }

  val updateBackupFormFields = mapping(
    "connection" -> ignored(None:Option[ConnectionName]),
    "name" -> ignored(""),
    "enabled" -> ignored(false),
    "transport" -> text
  )(Domain.apply)(Domain.unapply)

  val updateBackupForm = Form( updateBackupFormFields )

  def updateBackup(connection: ConnectionName, name: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      BackupAction(name) { implicit domainRequest =>
        updateBackupForm.bindFromRequest()(connectionRequest).fold(
          errors => {
            Logger.warn(s"Update backup domain form error")
            val relays  = domainRequest.backup.domain.findRelaysIfEnabled
            val aliases = domainRequest.backup.domain.findAliases
            val users   = domainRequest.backup.domain.findUsers
            val databaseDomains = domainRequest.backup.domain.findInDatabases
            BadRequest(views.html.domain.domaindetails( connection, None, Some(domainRequest.backup), relays, aliases, users, databaseDomains))
          },
          backup => {
            Backup(domainRequest.backup.domain.copy(transport=backup.transport)).update
            Logger.info(s"Domain updated: $name")
            Redirect(routes.DomainController.viewDomain(connection,name))
          }
        )
      }(connectionRequest)
    }(authRequest)
  }

}
