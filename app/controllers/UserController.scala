package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
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


object UserController extends Controller with DbController with FeatureToggler with UserInjector {

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

}
