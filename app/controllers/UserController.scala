package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import play.api.data._
import play.api.data.Forms._
import models._
import models.Environment.ConnectionName


class RequestWithUser[A](val user: User, request: Request[A]) extends WrappedRequest[A](request)

trait UserInjector {

  def UserAction(email: String) = new ActionBuilder[RequestWithUser] {
    def invokeBlock[A](request: Request[A], block: (RequestWithUser[A]) => Future[SimpleResult]) = {
      request match {
        case connectionRequest: RequestWithConnection[A] => {
          Users.findUser(connectionRequest.connection, email) match {
            case Some(user) => {
              Logger.warn(s"User $email found")
              block(new RequestWithUser(user, connectionRequest))
            }
            case None => {
              Logger.warn(s"User $email not found")
              val users = Users.findUsers(connectionRequest.connection)
              implicit val errorMessages = List(ErrorMessage("User not found"))
              implicit val user: Option[ApplicationUser] = None
              implicit val features = FeatureToggles.findFeatureToggles(connectionRequest.connection)
              Future.successful(
                NotFound(views.html.user.user( connectionRequest.connection, users) ) )
            }
          }
        }
        case _ => {
          Logger.error(s"Im so confused")
          Future.successful(InternalServerError)
        }
      }
    }
  }

}


object UserController extends Controller with DbController with FeatureToggler with UserInjector with DomainInjector with Secured {

  def user(connection: ConnectionName) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection) { implicit request =>
      val users = Users.findUsers(connection)
      Ok(views.html.user.user(connection,users))
    }(authRequest)
  }

  def viewUser(connection: ConnectionName, email: String) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
        val domain = userRequest.user.findDomain(connection)
        val alias = userRequest.user.findAlias(connection)
        Ok(views.html.user.edituser(connection,userRequest.user,domain,alias))
      }(connectionRequest)
    }(authRequest)
  }

  def disableUser(connection: ConnectionName, email: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
        userRequest.user.disable(connection)
        Logger.info(s"User disabled: $email")
        returnUrl match {
          case "orphan" => Redirect(routes.AliasController.orphan(connectionRequest.connection))
          case "edituser" => Redirect(routes.UserController.viewUser(connectionRequest.connection, email))
          case _ => Redirect(routes.UserController.user(connectionRequest.connection))
        }
      }(connectionRequest)
    }(authRequest)
  }

  def disable(connection: ConnectionName, domainName: String, email: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        UserAction(email) { implicit userRequest =>
          Logger.debug(s"User disabled soon: $email")
          userRequest.user.disable(connection)
          Logger.info(s"User disabled: $email")
          returnUrl match {
            case "removedomain" => Redirect(routes.DomainController.viewRemove(connectionRequest.connection, domainName))
            case _ => Redirect(routes.DomainController.viewDomain(connectionRequest.connection,domainName))
          }
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def enableUser(connection: ConnectionName, email: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
          Logger.debug(s"User enabled soon: $email")
        userRequest.user.enable(connection)
        Logger.info(s"User enabled: $email")
        returnUrl match {
          case "orphan" => Redirect(routes.AliasController.orphan(connectionRequest.connection))
          case "edituser" => Redirect(routes.UserController.viewUser(connectionRequest.connection, email))
          case _ => Redirect(routes.UserController.user(connectionRequest.connection))
        }
      }(connectionRequest)
    }(authRequest)
  }

  def enable(connection: ConnectionName, domainName: String, email: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        UserAction(email) { implicit userRequest =>
          userRequest.user.enable(connection)
          Logger.info(s"User enabled: $email")
          Redirect(routes.DomainController.viewDomain(connectionRequest.connection,domainName))
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  val userFormFields = mapping (
    "email" -> text,
    "name" -> text,
    "maildir" -> text,
    "passwordReset" -> ignored(true),
    "enabled" -> ignored(false)
  )(User.apply)(User.unapply)

  val userForm = Form( userFormFields )

  def viewAdd(connection: ConnectionName) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection) { implicit connectionRequest =>
      Ok(views.html.user.addUser( connection, None, userForm))
    }(authRequest)
  }


  def viewAddWithDomain(connection: ConnectionName, domainName: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName) { implicit domainRequest =>
        Ok(views.html.user.addUser( connection, Some(domainRequest.domainRequested), userForm))
      }(connectionRequest)
    }(authRequest)
  }


  def add(connection: ConnectionName) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection) { implicit connectionRequest =>
      userForm.bindFromRequest()(connectionRequest).fold(
        errors => {
          Logger.warn(s"Add user form error")
          BadRequest(views.html.user.addUser( connection, None, errors ))
        },
        user => {
          Users.findUser(connectionRequest.connection, user.email) match {
            case None => {
              Users.findUserByMaildir(connectionRequest.connection, user.maildir) match {
                case None if FeatureToggles.isAddEnabled(connectionRequest.connection) => {
                  user.save(connection)
                  Logger.info(s"User ${user.email} added")
                  Redirect(routes.UserController.user(connection))
                }
                case None => {
                  Logger.warn(s"Add feature not enabled")
                  implicit val errorMessages = List(ErrorMessage("Add feature not enabled"))
                  BadRequest(views.html.user.addUser( connection, None, userForm.fill(user)))
                }
                case Some(_) => {
                  Logger.warn(s"User maildir ${user.maildir} already exists")
                  implicit val errorMessages = List(ErrorMessage("User's maildir already exist"))
                  BadRequest(views.html.user.addUser( connection, None, userForm.fill(user)))
                }
              }
            }
            case Some(_) => {
              Logger.warn(s"User ${user.email} already exists")
              implicit val errorMessages = List(ErrorMessage("User already exist"))
              BadRequest(views.html.user.addUser( connection, None, userForm.fill(user)))
            }
          }
        }
      )
    }(authRequest)
  }

  def addWithDomain(connection: ConnectionName, domainName: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName) { domainRequest =>
        userForm.bindFromRequest()(domainRequest).fold(
          errors => {
            Logger.warn(s"Add user form error")
            BadRequest(views.html.user.addUser( connection, Some(domainRequest.domainRequested), errors ))
          },
          user => {
            Users.findUser(connectionRequest.connection, user.email) match {
              case None if FeatureToggles.isAddEnabled(connectionRequest.connection) => {
                user.save(connection)
                Logger.info(s"User ${user.email} added")
                Redirect(routes.DomainController.viewDomain(connection,domainName))
              }
              case None => {
                Logger.warn(s"Add feature not enabled")
                implicit val errorMessages = List(ErrorMessage("Add feature not enabled"))
                BadRequest(views.html.user.addUser( connection, Some(domainRequest.domainRequested), userForm.fill(user)))
              }
              case Some(_) => {
                Logger.warn(s"User ${user.email} already exists")
                implicit val errorMessages = List(ErrorMessage("User already exist"))
                BadRequest(views.html.user.addUser( connection, Some(domainRequest.domainRequested), userForm.fill(user)))
              }
            }
          }
        )
      }(connectionRequest)
    }(authRequest)
  }

  def remove(connection: ConnectionName, email: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
        userRequest.user.delete(connection)
        Logger.info(s"User removed: $email")
        returnUrl match {
          case "orphan" => Redirect(routes.AliasController.orphan(connectionRequest.connection))
          case _ => Redirect(routes.UserController.user(connectionRequest.connection))
        }
      }(connectionRequest)
    }(authRequest)
  }

  def removeDomainUser(connection: ConnectionName, domainName: String, email: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { domainRequest =>
        UserAction(email) { implicit userRequest =>
          userRequest.user.delete(connection)
          Logger.info(s"User removed: $email")
          Redirect(routes.DomainController.viewRemove(connectionRequest.connection,domainName))
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def resetPassword(connection: ConnectionName, email: String) = Authenticated.async { authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
        userRequest.user.resetPassword(connection)
        Logger.info(s"User password reset: $email")
        Redirect(routes.UserController.user(connectionRequest.connection))
      }(connectionRequest)
    }(authRequest)
  }

}
