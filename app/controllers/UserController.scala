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
    Ok(views.html.user.user(connection,users))
  }

  def edituser(connection: ConnectionName, email: String) = ConnectionAction(connection) {
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

}
