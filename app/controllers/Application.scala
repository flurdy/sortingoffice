package controllers

import javax.inject._
import scala.concurrent.{ExecutionContext, Future}
import play.api.mvc._
import play.api.data._
import play.api.data.Forms._
import play.api.Logging
import models._
import org.apache.pekko.stream.Materializer

class RequestWithConnection[A](val connection: String, request: Request[A]) extends WrappedRequest[A](request)

class DbController @Inject()(environment: Environment, bodyParsers: PlayBodyParsers)(implicit mat: Materializer) {
  implicit val databaseConnections: scala.List[(String, String)] = environment.databaseConnections

  implicit def connectionName[A](implicit request: RequestWithConnection[A]): String = request.connection

  def isValidConnection(connection: String): Boolean = databaseConnections.exists(_._1 == connection)

  def ConnectionAction(connection: String)(implicit ec: ExecutionContext): ActionBuilder[RequestWithConnection, AnyContent] =
    new ActionBuilder[RequestWithConnection, AnyContent] {
      override def parser: BodyParser[AnyContent] = bodyParsers.default
      override protected def executionContext: ExecutionContext = ec
      override def invokeBlock[A](request: Request[A], block: RequestWithConnection[A] => Future[Result]): Future[Result] = {
        if (isValidConnection(connection)) {
          block(new RequestWithConnection(connection, request))
        } else {
          val errorMessages = scala.List(ErrorMessage("Connection not found"))
          Future.successful(Results.NotFound(views.html.connections(databaseConnections)(errorMessages, None)))
        }
      }
    }
}

class FeatureToggler @Inject()(featureToggles: FeatureToggles) {
  implicit def featureTogglesForRequest[A](implicit request: RequestWithConnection[A]): FeatureToggleMap = featureToggles.findFeatureToggles(request.connection)
}

class AuthenticatedRequest[A](val username: String, request: Request[A]) extends WrappedRequest[A](request)
class AuthenticatedPossibleRequest[A](val username: Option[String], request: Request[A]) extends WrappedRequest[A](request)

class Secured @Inject()(cc: MessagesControllerComponents)(implicit ec: ExecutionContext) {
  def Authenticated: ActionBuilder[AuthenticatedRequest, AnyContent] = new ActionBuilder[AuthenticatedRequest, AnyContent] {
    override def parser: BodyParser[AnyContent] = cc.parsers.defaultBodyParser
    override protected def executionContext: ExecutionContext = ec
    override def invokeBlock[A](request: Request[A], block: AuthenticatedRequest[A] => Future[Result]): Future[Result] = {
      request.session.get("username") match {
        case Some(username) => block(new AuthenticatedRequest(username, request))
        case None =>
          val errorMessages = scala.List(ErrorMessage("Not authenticated"))
          Future.successful(Results.Forbidden(views.html.login(ApplicationController.loginForm)(errorMessages, None)))
      }
    }
  }

  def AuthenticatedPossible: ActionBuilder[AuthenticatedPossibleRequest, AnyContent] = new ActionBuilder[AuthenticatedPossibleRequest, AnyContent] {
    override def parser: BodyParser[AnyContent] = cc.parsers.defaultBodyParser
    override protected def executionContext: ExecutionContext = ec
    override def invokeBlock[A](request: Request[A], block: AuthenticatedPossibleRequest[A] => Future[Result]): Future[Result] = {
      block(new AuthenticatedPossibleRequest(request.session.get("username"), request))
    }
  }

  implicit def currentUser[A](implicit request: AuthenticatedRequest[A]): scala.Option[ApplicationUser] = scala.Some(ApplicationUser(request.username))
  implicit def currentPossibleUser[A](implicit request: AuthenticatedPossibleRequest[A]): scala.Option[ApplicationUser] = request.username.map(ApplicationUser(_))
}

object ApplicationController {
  val loginFields = mapping(
    "username" -> text,
    "password" -> text
  )((username, password) => LoginDetails(username, password))((ld: LoginDetails) => Some((ld.username, ld.password)))
  val loginForm = Form(loginFields)
}

@Singleton
class ApplicationController @Inject()(
  cc: MessagesControllerComponents,
  environment: Environment,
  applicationUsers: ApplicationUsers,
  bodyParsers: PlayBodyParsers
)(implicit ec: ExecutionContext, mat: Materializer)
  extends AbstractController(cc)
  with Logging {

  val dbController = new DbController(environment, bodyParsers)
  val secured = new Secured(cc)

  import dbController._
  import secured._

  def index = AuthenticatedPossible { implicit authRequest: AuthenticatedPossibleRequest[AnyContent] =>
    implicit val errorMessages: scala.List[ErrorMessage] = scala.List.empty
    implicit val currentUser: scala.Option[ApplicationUser] = currentPossibleUser
    databaseConnections.size match {
      case 0 => NotFound(views.html.connections(scala.List.empty))
      case 1 => Redirect(routes.ApplicationController.connectionIndex(databaseConnections.head._1))
      case _ => Ok(views.html.connections(databaseConnections))
    }
  }

  def connectionIndex(connection: String) = AuthenticatedPossible.async { implicit authRequest: AuthenticatedPossibleRequest[AnyContent] =>
    implicit val errorMessages: scala.List[ErrorMessage] = scala.List.empty
    implicit val currentUser: scala.Option[ApplicationUser] = currentPossibleUser
    ConnectionAction(connection).invokeBlock(authRequest, { request =>
      Future.successful(Ok(views.html.index(connection)))
    })
  }

  def about = AuthenticatedPossible { implicit authRequest: AuthenticatedPossibleRequest[AnyContent] =>
    implicit val errorMessages: scala.List[ErrorMessage] = scala.List.empty
    implicit val currentUser: scala.Option[ApplicationUser] = currentPossibleUser
    Ok(views.html.about())
  }

  def contact = AuthenticatedPossible { implicit authRequest: AuthenticatedPossibleRequest[AnyContent] =>
    implicit val errorMessages: scala.List[ErrorMessage] = scala.List.empty
    implicit val currentUser: scala.Option[ApplicationUser] = currentPossibleUser
    Ok(views.html.contact())
  }

  def viewLogin = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: scala.List[ErrorMessage] = scala.List.empty
    implicit val currentUser: scala.Option[ApplicationUser] = None
    Ok(views.html.login(ApplicationController.loginForm)).withNewSession
  }

  def login = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: scala.List[ErrorMessage] = scala.List.empty
    implicit val currentUser: scala.Option[ApplicationUser] = None
    ApplicationController.loginForm.bindFromRequest().fold(
      errors => {
        logger.warn(s"Login form error")
        BadRequest(views.html.login(errors))
      },
      loginDetails => {
        applicationUsers.authenticateLoginDetails(loginDetails) match {
          case Some(applicationUser) =>
            Redirect(routes.ApplicationController.index).withSession("username" -> loginDetails.username)
          case None =>
            logger.warn(s"Authentication failed. Either the user does not exist or the password is incorrect")
            val errorMessages = scala.List(ErrorMessage(
              "Authentication failed. Either the user does not exist or the password is incorrect"))
            BadRequest(views.html.login(ApplicationController.loginForm.fill(loginDetails))(errorMessages, None))
        }
      }
    )
  }

  def logout = Action {
    Redirect(routes.ApplicationController.index).withNewSession
  }

  val registerFields = mapping(
    "username" -> text,
    "password" -> text,
    "confirmPassword" -> text
  )((username, password, confirmPassword) => RegisterDetails(username, password, confirmPassword))((rd: RegisterDetails) => Some((rd.username, rd.password, rd.confirmPassword))) verifying ("Passwords does not match", fields => fields match {
    case RegisterDetails(_, password, confirmPassword) => password == confirmPassword
  })

  val registerForm = Form(registerFields)

  def viewRegister = AuthenticatedPossible { implicit authRequest: AuthenticatedPossibleRequest[AnyContent] =>
    implicit val errorMessages: scala.List[ErrorMessage] = scala.List.empty
    implicit val currentUser: scala.Option[ApplicationUser] = currentPossibleUser
    Ok(views.html.register(registerForm))
  }

  def register = AuthenticatedPossible { implicit authRequest: AuthenticatedPossibleRequest[AnyContent] =>
    implicit val errorMessages: scala.List[ErrorMessage] = scala.List.empty
    implicit val currentUser: scala.Option[ApplicationUser] = currentPossibleUser
    registerForm.bindFromRequest().fold(
      errors => {
        logger.warn(s"Register form error")
        BadRequest(views.html.register(errors))
      },
      registerDetails => {
        applicationUsers.register(registerDetails)
        Ok(views.html.registered(registerForm)).withNewSession
      }
    )
  }
}
