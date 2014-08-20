package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import models._
import models.Environment.ConnectionName


object RelayController extends Controller with DbController{

  def disable(connection: ConnectionName, recipient: String) = ConnectionAction(connection) {
    Relays.findRelay(connection,recipient) match {
      case Some(relay) => {
        relay.disable(connection)
        Redirect(routes.AliasController.alias(connection))
      }
      case None => {
        Logger.warn(s"Relay $recipient not found")
        implicit val errorMessages = List(ErrorMessage("Relay not found"))
        NotFound(views.html.alias.alias(connection))
      }
    }
  }
  
  def enable(connection: ConnectionName, recipient: String) = ConnectionAction(connection) {
    Relays.findRelay(connection,recipient) match {
      case Some(relay) => {
        relay.enable(connection)
        Redirect(routes.AliasController.alias(connection))
      }
      case None => {
        Logger.warn(s"Relay $recipient not found")
        implicit val errorMessages = List(ErrorMessage("Relay not found"))
        NotFound(views.html.alias.alias(connection))
      }
    }
  }


}
