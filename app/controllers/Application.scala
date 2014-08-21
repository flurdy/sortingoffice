package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
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


object Application extends Controller with DbController {

  def index = Action {
    databaseConnections.size match {
      case 0 => NotFound(views.html.connections(List.empty))
      case 1 => Redirect(routes.Application.connectionIndex(databaseConnections.head._1))
      case _ => Ok(views.html.connections(databaseConnections))
    }
  }

  def connectionIndex(connection: ConnectionName) = ConnectionAction(connection) {
      Ok(views.html.index(connection))
  }

  def about = Action {
    Ok(views.html.about())
  }

  def contact = Action {
    Ok(views.html.contact())
  }

}
