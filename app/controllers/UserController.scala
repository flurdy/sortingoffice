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
              block(new RequestWithUser(user, connectionRequest))
            }
            case None => {
              Logger.warn(s"User $email not found")
              val users = Users.findUsers(connectionRequest.connection)
              implicit val errorMessages = List(ErrorMessage("User not found"))
              Future.successful(
                NotFound(views.html.user.user( connectionRequest.connection, users)(
                  errorMessages,FeatureToggles.findFeatureToggles(connectionRequest.connection) ) ) )
            }
          }
        }
        case _ => Future.successful(InternalServerError)
      }
    }
  }

}


object UserController extends Controller with DbController with FeatureToggler with UserInjector with DomainInjector {

  def user(connection: ConnectionName) = ConnectionAction(connection) { implicit request =>
    val users = Users.findUsers(connection)
    Ok(views.html.user.user(connection,users))
  }

  def edituser(connection: ConnectionName, email: String) = {
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
          Ok(views.html.user.edituser(connection,userRequest.user))
      }(connectionRequest)
    }
  }

  def disable(connection: ConnectionName, email: String) = {
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
        userRequest.user.disable(connection)
        Redirect(routes.UserController.user(connectionRequest.connection))
      }(connectionRequest)
    }
  }

  def enable(connection: ConnectionName, email: String) = {
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
        userRequest.user.enable(connection)
        Redirect(routes.UserController.user(connectionRequest.connection))
      }(connectionRequest)
    }
  }

  val userFormFields = mapping (
    "email" -> text,
    "name" -> text,
    "maildir" -> text,
    "passwordReset" -> ignored(true),
    "enabled" -> ignored(false)
  )(User.apply)(User.unapply)

  val userForm = Form( userFormFields )

  def viewAdd(connection: ConnectionName) = ConnectionAction(connection) { implicit connectionRequest =>
    Ok(views.html.user.addUser( connection, None, userForm))
  }


  def viewAddWithDomain(connection: ConnectionName, domainName: String) = ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName) { implicit domainRequest =>
        Ok(views.html.user.addUser( connection, Some(domainRequest.domainRequested), userForm))
      }(connectionRequest)
  }


  def add(connection: ConnectionName) = ConnectionAction(connection) { implicit connectionRequest =>
    userForm.bindFromRequest.fold(
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
  }

  def addWithDomain(connection: ConnectionName, domainName: String) = ConnectionAction(connection).async { implicit connectionRequest =>
    DomainAction(domainName) { domainRequest =>
      userForm.bindFromRequest.fold(
        errors => {
          Logger.warn(s"Add user form error")
          BadRequest(views.html.user.addUser( connection, Some(domainRequest.domainRequested), errors ))
        },
        user => {
          Users.findUser(connectionRequest.connection, user.email) match {
            case None if FeatureToggles.isAddEnabled(connectionRequest.connection) => {
              user.save(connection)
              Logger.info(s"User ${user.email} added")
              Redirect(routes.DomainController.alias(connection,domainName))
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
  }

  def remove(connection: ConnectionName, email: String) = {
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
        userRequest.user.delete(connection)
        Redirect(routes.UserController.user(connectionRequest.connection))
      }(connectionRequest)
    }
  }

  def resetPassword(connection: ConnectionName, email: String) = {
    ConnectionAction(connection).async { implicit connectionRequest =>
      UserAction(email) { implicit userRequest =>
        userRequest.user.resetPassword(connection)
        Redirect(routes.UserController.user(connectionRequest.connection))
      }(connectionRequest)
    }
  }


}
