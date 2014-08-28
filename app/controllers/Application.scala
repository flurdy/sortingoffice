package controllers


import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Security._
import play.api.mvc.Results._
import play.api.data._
import play.api.data.Forms._
import models._
import models.Environment.ConnectionName


class RequestWithConnection[A](val connection: ConnectionName, request: Request[A]) extends WrappedRequest[A](request)

trait DbController {

  implicit val databaseConnections: List[(String,String)] = Environment.databaseConnections

  implicit def connectionName[A](implicit request: RequestWithConnection[A]): ConnectionName = request.connection

  def isValidConnection(connection: ConnectionName): Boolean = databaseConnections.exists( _._1 == connection )

  def ConnectionAction(connection: ConnectionName) = new ActionBuilder[RequestWithConnection] {
    def invokeBlock[A](request: Request[A], block: (RequestWithConnection[A]) => Future[SimpleResult]) = {
      if( isValidConnection(connection) ){
        block(new RequestWithConnection(connection, request))
      } else {
        implicit val errorMessages = List(ErrorMessage("Connection not found"))
        implicit val user: Option[ApplicationUser] = None
        Future.successful(
          NotFound(views.html.connections(databaseConnections))
        )
      }
    }
  }

}


trait FeatureToggler {

  implicit def featureToggles[A](implicit request: RequestWithConnection[A]): FeatureToggleMap = FeatureToggles.findFeatureToggles(request.connection)

}


class AuthenticatedRequest[A](val username: String, request: Request[A]) extends WrappedRequest[A](request)

class AuthenticatedPossibleRequest[A](val username: Option[String], request: Request[A]) extends WrappedRequest[A](request)

trait Secured {

  def Authenticated = new ActionBuilder[AuthenticatedRequest] {
    def invokeBlock[A](request: Request[A], block: (AuthenticatedRequest[A]) => Future[SimpleResult]) = {
      request.session.get("username") match {
        case Some(username) => {
          block(new AuthenticatedRequest(username, request))
        }
        case None => {
          implicit val errorMessages = List(ErrorMessage("Not authenticated"))
          implicit val user: Option[ApplicationUser] = None
          Future.successful(Forbidden(views.html.login(Application.loginForm)))
        }
      }
    }
  }

  def AuthenticatedPossible = new ActionBuilder[AuthenticatedPossibleRequest] {
    def invokeBlock[A](request: Request[A], block: (AuthenticatedPossibleRequest[A]) => Future[SimpleResult]) = {
      block(new AuthenticatedPossibleRequest(request.session.get("username"), request))
    }
  }

  implicit def currentUser[A](implicit request: AuthenticatedRequest[A]): Option[ApplicationUser] = {        
    Some(ApplicationUser(request.username))
  }

  implicit def currentPossibleUser[A](implicit request: AuthenticatedPossibleRequest[A]): Option[ApplicationUser] = {      
    request.username.map( ApplicationUser(_) )
  }

}


object Application extends Controller with DbController with Secured {

  def index = AuthenticatedPossible { implicit authRequest =>
    databaseConnections.size match {
      case 0 => NotFound(views.html.connections(List.empty))
      case 1 => Redirect(routes.Application.connectionIndex(databaseConnections.head._1))
      case _ => Ok(views.html.connections(databaseConnections))
    }
  }

  def connectionIndex(connection: ConnectionName) = AuthenticatedPossible.async { implicit authRequest =>
    ConnectionAction(connection) { request =>
      Ok(views.html.index(connection))
    }(authRequest)
  }

  def about = AuthenticatedPossible { implicit authRequest =>
    Ok(views.html.about())
  }

  def contact = AuthenticatedPossible { implicit authRequest =>
    Ok(views.html.contact())
  }

  val loginFields = mapping (
    "username" -> text,
    "password" -> text
  )(LoginDetails.apply)(LoginDetails.unapply)

  val loginForm = Form( loginFields )

  def viewLogin = Action {
    Ok(views.html.login(loginForm)).withNewSession
  }

  def login = Action { implicit request =>
    loginForm.bindFromRequest.fold(
      errors => {
        Logger.warn(s"Login form error")
        BadRequest(views.html.login(errors))
      },
      loginDetails => {
        ApplicationUsers.authenticateLoginDetails(loginDetails) match {
          case Some(applicationUser) => {

            Redirect(routes.Application.index()).withSession("username" -> loginDetails.username)

          }
          case None => {
            Logger.warn(s"Authentication failed. Either the user does not exist or the password is incorrect")
            implicit val errorMessages = List(ErrorMessage(
                "Authentication failed. Either the user does not exist or the password is incorrect"))
            BadRequest(views.html.login(loginForm.fill(loginDetails)))
          }
        }
      }
    )
  }

  def logout = Action {
    Redirect(routes.Application.index()).withNewSession
  }

  val registerFields = mapping (
    "username" -> text,
    "password" -> text,
    "confirmPassword" -> text
  )(RegisterDetails.apply)(RegisterDetails.unapply) verifying("Password does not match", fields => fields match {
    case registerDetails => registerDetails.password == registerDetails.confirmPassword
  })

  val registerForm = Form( registerFields )

  def viewRegister = AuthenticatedPossible { implicit authRequest =>
    Ok(views.html.register(registerForm))
  }

  def register = AuthenticatedPossible { implicit authRequest =>
    registerForm.bindFromRequest.fold(
      errors => {
        Logger.warn(s"Register form error")
        BadRequest(views.html.register(errors))
      },
      registerDetails => {
        ApplicationUsers.register(registerDetails)
        Ok(views.html.registered(registerForm)).withNewSession
      }
    )
  }
}
