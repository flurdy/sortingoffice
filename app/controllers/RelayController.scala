package controllers

import scala.concurrent.Future
import play.api._
import play.api.mvc._
import play.api.mvc.Results._
import play.api.data._
import play.api.data.Forms._
import models._
import models.Environment.ConnectionName

class RequestWithRelay[A](val relay: Relay, request: Request[A]) extends WrappedRequest[A](request)

trait RelayInjector {

  def RelayAction(recipient: String) = new ActionBuilder[RequestWithRelay] {
    def invokeBlock[A](request: Request[A], block: (RequestWithRelay[A]) => Future[SimpleResult]) = {
      request match {
        case connectionRequest: RequestWithConnection[A] => {
          Relays.findRelay(connectionRequest.connection, recipient) match {
            case Some(relay) => {
              block(new RequestWithRelay(relay, connectionRequest))
            }
            case None => {
              Logger.warn(s"Relay $recipient not found")
              implicit val errorMessages = List(ErrorMessage("Relay not found"))
              Future.successful( NotFound(views.html.alias.alias(connectionRequest.connection)(errorMessages) ) )
            }
          }
        }
        case _ => Future.successful(InternalServerError)
      }
    }
  }

}


object RelayController extends Controller with DbController with RelayInjector with DomainInjector with FeatureToggler {

  def disable(connection: ConnectionName, domainName: String, recipient: String) = ConnectionAction(connection).async { implicit connectionRequest =>
    DomainOrBackupAction(domainName).async { implicit domainRequest =>
      RelayAction(recipient) { implicit request =>
        request.relay.disable(connection)
        Redirect(routes.DomainController.alias(connection,domainName))
      }(connectionRequest)
    }(connectionRequest)
  }

  def disableRelay(connection: ConnectionName, recipient: String) = ConnectionAction(connection).async { implicit connectionRequest =>
    RelayAction(recipient) { implicit request =>
      request.relay.disable(connection)
      Redirect(routes.AliasController.orphan(connection))
    }(connectionRequest)
  }

  def enable(connection: ConnectionName, domainName: String, recipient: String) =ConnectionAction(connection).async { implicit connectionRequest =>
    DomainOrBackupAction(domainName).async { implicit domainRequest =>
      RelayAction(recipient) { implicit request =>
        request.relay.enable(connection)
        Redirect(routes.DomainController.alias(connection,domainName))
      }(connectionRequest)
    }(connectionRequest)
  }

  val relayFormFields = mapping (
    "recipient" -> text,
    "enabled" -> ignored(false),
    "status" -> text
  )(Relay.apply)(Relay.unapply)

  val relayForm = Form( relayFormFields )


  def viewAdd(connection: ConnectionName, domainName: String) = ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName) { implicit domainRequest =>
        Ok(views.html.relay.addRelay( connection, domainRequest.domainRequested, relayForm))
      }(connectionRequest)
  }

  def viewAddCatchAll(connection: ConnectionName, domainName: String) = ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName) { implicit domainRequest =>
        Ok(views.html.relay.addRelay( connection, domainRequest.domainRequested, relayForm.fill(Relay(s"@$domainName",false,"OK"))))
      }(connectionRequest)
  }

  def add(connection: ConnectionName, domainName: String) = ConnectionAction(connection).async { implicit connectionRequest =>
    DomainOrBackupAction(domainName) { domainRequest =>
      relayForm.bindFromRequest.fold(
        errors => {
          Logger.warn(s"Add relay form error")
          BadRequest(views.html.relay.addRelay( connection, domainRequest.domainRequested, errors ))
        },
        relay => {
          Relays.findRelay(connectionRequest.connection, relay.recipient) match {
            case None if FeatureToggles.isAddEnabled(connectionRequest.connection) => {
              relay.save(connection)
              Redirect(routes.DomainController.alias(connection,domainName))
            }
            case None => {
              Logger.warn(s"Add feature not enabled")
              implicit val errorMessages = List(ErrorMessage("Add feature not enabled"))
              BadRequest(views.html.relay.addRelay( connection, domainRequest.domainRequested, relayForm.fill(relay)))
            }
            case Some(_) => {
              Logger.warn(s"Relay ${relay.recipient} already exists")
              implicit val errorMessages = List(ErrorMessage("Relay already exist"))
              BadRequest(views.html.relay.addRelay( connection, domainRequest.domainRequested, relayForm.fill(relay)))
            }
          }
        }
      )
    }(connectionRequest)
  }

  def remove(connection: ConnectionName, domainName: String, recipient: String) = ConnectionAction(connection).async { implicit connectionRequest =>
    DomainOrBackupAction(domainName).async { implicit domainRequest =>
      RelayAction(recipient) { implicit request =>
        request.relay.delete(connection)
        Redirect(routes.DomainController.alias(connection,domainName))
      }(connectionRequest)
    }(connectionRequest)
  }

  def removeRelay(connection: ConnectionName, recipient: String) = ConnectionAction(connection).async { implicit connectionRequest =>
    RelayAction(recipient) { implicit request =>
      request.relay.delete(connection)
      Redirect(routes.AliasController.orphan(connection))
    }(connectionRequest)
  }

}
