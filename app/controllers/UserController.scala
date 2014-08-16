package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName




object UserController extends Controller with DbController {

  def user(connection: ConnectionName) = ConnectionAction(connection) {
    val users = Users.findUsers(connection)
    val toggle = FeatureToggles.isToggleEnabled(connection)
    Ok(views.html.user.user(connection,users,toggle))
  }

  def edituser(connection: ConnectionName, email: String) = ConnectionAction(connection) {
    val toggle = FeatureToggles.isToggleEnabled(connection)
    Users.findUser(connection, email) match {
      case Some(user) => Ok(views.html.user.edituser(connection,user,toggle))
      case None => {
        Logger.warn(s"User $email not found")
        val users = Users.findUsers(connection)
        implicit val errorMessages = List(ErrorMessage("User not found"))
        NotFound(views.html.user.user( connection, users, toggle))
      }
    }
  }

  def disable(connection: ConnectionName, email: String) = ConnectionAction(connection) {
    Users.findUser(connection, email) match {
      case Some(user) => {
        user.disable(connection)
        Redirect(routes.UserController.user(connection))
      }
      case None => {
        Logger.warn(s"User $email not found")
        val users = Users.findUsers(connection)
        val toggle = FeatureToggles.isToggleEnabled(connection)
        implicit val errorMessages = List(ErrorMessage("User not found"))
        NotFound(views.html.user.user( connection, users, toggle))
      }
    }
  }

  def enable(connection: ConnectionName, email: String) = ConnectionAction(connection) {
    Users.findUser(connection, email) match {
      case Some(user) => {
        user.enable(connection)
        Redirect(routes.UserController.user(connection))
      }
      case None => {
        Logger.warn(s"User $email not found")
        val users = Users.findUsers(connection)
        val toggle = FeatureToggles.isToggleEnabled(connection)
        implicit val errorMessages = List(ErrorMessage("User not found"))
        NotFound(views.html.user.user( connection, users, toggle))
      }
    }
  }


}
