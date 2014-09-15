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
              Logger.debug(s"Alias ${alias.mail} found")
              block(new RequestWithAlias(alias, connectionRequest))
            }
            case None => {
              Logger.warn(s"Alias $email not found")
              implicit val errorMessages = List(ErrorMessage("Alias not found"))
              Future.successful( NotFound(views.html.alias.alias(connectionRequest.connection)(errorMessages,None) ) )
            }
          }
        }
        case _ => {
          Logger.error(s"Alias $email created server error")
          implicit val errorMessages = List(ErrorMessage("Internal error"))
          Future.successful(InternalServerError(views.html.connections(Nil)(errorMessages,None) ) )
        }
      }
    }
  }

}

object AliasController extends Controller with DbController with FeatureToggler with AliasInjector with DomainInjector with UserInjector with Secured {

  def alias(connection: ConnectionName) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection) {
      Ok(views.html.alias.alias(connection))
    }(authRequest)
  }

  def catchAll(connection: ConnectionName) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      val allDomains: List[Domain] = Domains.findDomains(connection)
      val catchAllAliases: (List[(Domain,Alias)],List[(Domain,Option[Alias])]) =
          Aliases.findCatchAllDomains(connection,allDomains)
      val allBackups: List[Domain] = Domains.findBackupDomains(connection).map(_.domain)
      val catchAllRelays: Option[(List[(Domain,Relay)],List[(Domain,Option[Relay])])] =
          Relays.findCatchAllDomainsIfEnabled(connection,allDomains)
      val catchAllBackupRelays: Option[(List[(Backup,Relay)],List[(Backup,Option[Relay])])] =
          Relays.findCatchAllBackupsIfEnabled(connection,allBackups)
      Ok(views.html.alias.catchall( connection,
                                    catchAllAliases._1,
                                    catchAllAliases._2,
                                    catchAllRelays.map(_._1),
                                    catchAllBackupRelays.map(_._1),
                                    catchAllRelays.map(_._2),
                                    catchAllBackupRelays.map(_._2) ) )
    }(authRequest)
  }

  def common(connection: ConnectionName) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      val domains = Domains.findDomains(connection)
      val requiredAliases: List[(Domain,Map[String,Boolean])] = Aliases.findRequiredAndCommonAliases(domains)
      val requiredRelays: Option[List[(Domain,Map[String,Boolean])]] = Relays.findRequiredAndCommonRelaysIfEnabled(connection,domains)
      Ok(views.html.alias.common( connection, requiredAliases, requiredRelays ))
    }(authRequest)
  }

  def crossDomain(connection: ConnectionName) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      val aliases = Aliases.customAliases
      val relayDomains = Domains.findDomains(connection)
      val customAliases: List[(Domain, (Map[String,Boolean], Option[Map[String,Boolean]]))] = relayDomains.map{ d =>
        ( d, d.findCustomAliasesAndRelays )
      }
      Ok(views.html.alias.cross(connection, aliases, customAliases) )
    }(authRequest)
  }

  def orphan(connection: ConnectionName) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      val domains = Domains.findDomains(connection)
      val backups = Domains.findBackupDomains(connection).map(_.domain)
      val aliases = Aliases.findOrphanAliases(connection,domains)
      val relays = Relays.findOrphanRelays(connection,domains++backups)
      val users = Users.findOrphanUsers(connection,domains)
      Ok(views.html.alias.orphan(connection, aliases, relays, users) )
    }(authRequest)
  }

  def all(connection: ConnectionName) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      val aliases = Aliases.findAllAliases(connection)
      Ok(views.html.alias.all(connection, aliases) )
    }(authRequest)
  }

  def disable(connection: ConnectionName, domainName: String, email: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        AliasAction(email) { implicit aliasRequest =>
          aliasRequest.alias.disable(connection)
          Logger.info(s"Alias ${email} disabled")
          returnUrl match {
            case "catchall" => Redirect(routes.AliasController.catchAll(connection))
            case "aliasdetails" => Redirect(routes.AliasController.viewAlias(connection,domainName,email))
            case "userdetails" => Redirect(routes.UserController.viewUser(connection,email))
            case "removedomain" => Redirect(routes.DomainController.viewRemove(connection,domainName))
            case "allalias" => Redirect(routes.AliasController.all(connection))
            case _ => Redirect(routes.DomainController.viewDomain(connection,domainName))
          }
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def disableOrphanAlias(connection: ConnectionName, email: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      AliasAction(email) { implicit aliasRequest =>
        aliasRequest.alias.disable(connection)
        Logger.info(s"Alias ${email} disabled")
        returnUrl match {
          case "userdetails" => Redirect(routes.UserController.viewUser(connection,email))
          case "allalias" => Redirect(routes.AliasController.all(connection))
          case _ => Redirect(routes.AliasController.orphan(connection))
        }
      }(connectionRequest)
    }(authRequest)
  }

  def enable(connection: ConnectionName, domainName: String, email: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        AliasAction(email) { implicit aliasRequest =>
          aliasRequest.alias.enable(connection)
          Logger.info(s"Alias ${email} enabled")
          returnUrl match {
            case "catchall" => Redirect(routes.AliasController.catchAll(connection))
            case "aliasdetails" => Redirect(routes.AliasController.viewAlias(connection,domainName,email))
            case "userdetails" => Redirect(routes.UserController.viewUser(connection,email))
            case "allalias" => Redirect(routes.AliasController.all(connection))
            case _ => Redirect(routes.DomainController.viewDomain(connection,domainName))
          }
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }


  val aliasFormFields = mapping (
    "mail" -> text,
    "destination" -> text,
    "enabled" -> ignored(false)
  )(Alias.apply)(Alias.unapply)

  val aliasForm = Form( aliasFormFields )

  def viewAdd(connection: ConnectionName, domainName: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName) { implicit domainRequest =>
        Ok(views.html.alias.addAlias( connection, domainRequest.domainRequested, aliasForm, "domaindetails" ))
      }(connectionRequest)
    }(authRequest)
  }

  def viewAddCatchAll(connection: ConnectionName, domainName: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName) { implicit domainRequest =>
        Ok(views.html.alias.addAlias( connection, domainRequest.domainRequested, aliasForm.fill(Alias(s"@$domainName","",false)), "catchall" ))
      }(connectionRequest)
    }(authRequest)
  }

  def add(connection: ConnectionName, domainName: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName) { implicit domainRequest =>
        aliasForm.bindFromRequest()(domainRequest).fold(
          errors => {
            Logger.warn(s"Add alias form error")
            BadRequest(views.html.alias.addAlias( connection, domainRequest.domainRequested, errors, returnUrl))
          },
          aliasToAdd => {
            Aliases.findAlias(connectionRequest.connection, aliasToAdd.mail) match {
              case None if FeatureToggles.isAddEnabled(connectionRequest.connection) => {
                aliasToAdd.save(connection)
                Logger.info(s"Alias ${aliasToAdd.mail} added")
                returnUrl match {
                  case "catchall" => Redirect(routes.AliasController.catchAll(connection))
                  case _ => Redirect(routes.DomainController.viewDomain(connection,domainName))
                }
              }
              case None => {
                Logger.warn(s"Add feature not enabled")
                implicit val errorMessages = List(ErrorMessage("Add feature not enabled"))
                BadRequest(views.html.alias.addAlias( connection, domainRequest.domainRequested, aliasForm.fill(aliasToAdd), returnUrl))
              }
              case Some(_) => {
                Logger.warn(s"Alias ${aliasToAdd.mail} already exists")
                implicit val errorMessages = List(ErrorMessage("Alias already exist"))
                BadRequest(views.html.alias.addAlias( connection, domainRequest.domainRequested, aliasForm.fill(aliasToAdd), returnUrl))
              }
            }
          }
        )
      }(connectionRequest)
    }(authRequest)
  }

  def addUserAlias(connection: ConnectionName, domainName: String, email: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        UserAction(email) { implicit userRequest =>
          Aliases.findAlias(connectionRequest.connection, email) match {
            case None => {
              Aliases.createAlias(email).save(connection)
              Logger.info(s"Alias ${email} added")
              Redirect(routes.UserController.viewUser(connection,email))
            }
            case Some(alias) => {
              Logger.warn(s"Alias ${email} already exists")
              implicit val errorMessages = List(ErrorMessage("Alias already exist"))
              BadRequest(views.html.user.edituser( connection, userRequest.user, Some(domainRequest.domainRequested), Some(alias), UserController.updateUserForm ))
            }
          }
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }


  def remove(connection: ConnectionName, domainName: String, email: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        AliasAction(email) { implicit aliasRequest =>
          aliasRequest.alias.delete(connection)
          Logger.info(s"Alias ${email} deleted")
          returnUrl match {
            case "catchall" => Redirect(routes.AliasController.catchAll(connection))
            case "removedomain" => Redirect(routes.DomainController.viewRemove(connection,domainName))
            case _ => Redirect(routes.DomainController.viewDomain(connection,domainName))
          }
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def removeAlias(connection: ConnectionName, email: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      AliasAction(email) { implicit aliasRequest =>
        aliasRequest.alias.delete(connection)
        Logger.info(s"Alias ${email} deleted")
        Redirect(routes.AliasController.orphan(connection))
      }(connectionRequest)
    }(authRequest)
  }

  def viewAlias(connection: ConnectionName, domainName: String, email: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        AliasAction(email) { implicit aliasRequest =>
          val relays = Relays.findRelaysForAliasIfEnabled(connection, domainRequest.domainRequested, aliasRequest.alias)
          Ok(views.html.alias.aliasdetails(connection,domainRequest.domainRequested,aliasRequest.alias,relays))
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  val updateAliasFormFields = mapping (
    "mail" -> ignored(""),
    "destination" -> text,
    "enabled" -> ignored(false)
  )(Alias.apply)(Alias.unapply)

  val updateAliasForm = Form( updateAliasFormFields )

  def updateAlias(connection: ConnectionName, domainName: String, email: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        AliasAction(email) { implicit aliasRequest =>
          updateAliasForm.bindFromRequest()(domainRequest).fold(
            errors => {
              Logger.warn(s"Update alias form error")
              val relays = Relays.findRelaysForAliasIfEnabled(connection, domainRequest.domainRequested, aliasRequest.alias)
              BadRequest(views.html.alias.aliasdetails(connection,domainRequest.domainRequested,aliasRequest.alias,relays))
            },
            alias => {
              aliasRequest.alias.copy(destination=alias.destination).update(connection)
              Logger.info(s"Alias ${email} updated")
              Redirect(routes.AliasController.viewAlias(connection,domainName,email))
            }
          )
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

}

