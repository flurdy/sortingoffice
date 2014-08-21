package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName



object UserController extends Controller with DbController with FeatureToggler {

  def user(connection: ConnectionName) = ConnectionAction(connection) { implicit request =>
    val users = Users.findUsers(connection)
    Ok(views.html.user.user(connection,users))
  }

  def edituser(connection: ConnectionName, email: String) = ConnectionAction(connection) { implicit request =>
    Users.findUser(connection, email) match {
      case Some(user) => Ok(views.html.user.edituser(connection,user))
      case None => {
        Logger.warn(s"User $email not found")
        val users = Users.findUsers(connection)
        implicit val errorMessages = List(ErrorMessage("User not found"))
        NotFound(views.html.user.user( connection, users))
      }
    }
  }

  def disable(connection: ConnectionName, email: String) = ConnectionAction(connection) { implicit request =>
    Users.findUser(connection, email) match {
      case Some(user) => {
        user.disable(connection)
        Redirect(routes.UserController.user(connection))
      }
      case None => {
        Logger.warn(s"User $email not found")
        val users = Users.findUsers(connection)
        implicit val errorMessages = List(ErrorMessage("User not found"))
        NotFound(views.html.user.user( connection, users))
      }
    }
  }

  def enable(connection: ConnectionName, email: String) = ConnectionAction(connection) { implicit request =>
    Users.findUser(connection, email) match {
      case Some(user) => {
        user.enable(connection)
        Redirect(routes.UserController.user(connection))
      }
      case None => {
        Logger.warn(s"User $email not found")
        val users = Users.findUsers(connection)
        implicit val errorMessages = List(ErrorMessage("User not found"))
        NotFound(views.html.user.user( connection, users))
      }
    }
  }

}
