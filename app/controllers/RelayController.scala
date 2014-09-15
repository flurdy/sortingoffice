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
              Logger.debug(s"Relay ${relay.recipient} found")
              block(new RequestWithRelay(relay, connectionRequest))
            }
            case None => {
              Logger.warn(s"Relay $recipient not found")
              implicit val errorMessages = List(ErrorMessage("Relay not found"))
              implicit val user: Option[ApplicationUser] = None
              Future.successful( NotFound(views.html.alias.alias(connectionRequest.connection) ) )
            }
          }
        }
        case _ => Future.successful(InternalServerError)
      }
    }
  }

}


object RelayController extends Controller with DbController with RelayInjector with DomainInjector with AliasInjector with FeatureToggler with Secured {

  def disable(connection: ConnectionName, domainName: String, recipient: String, returnUrl: String) = Authenticated.async { authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName).async { implicit domainRequest =>
        RelayAction(recipient) { implicit request =>
          request.relay.disable(connection)
          returnUrl match {
            case "catchall" => Redirect(routes.AliasController.catchAll(connection))
            case "removedomain" => Redirect(routes.DomainController.viewRemove(connection,domainName))
            case "relaydetails" => Redirect(routes.RelayController.viewRelay(connection,domainName,recipient))
            case _ => Redirect(routes.DomainController.viewDomain(connection,domainName))
          }
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def disableAliasRelay(connection: ConnectionName, domainName: String, email: String, recipient: String, returnUrl: String) = Authenticated.async { authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        AliasAction(email).async { implicit aliasRequest =>
          RelayAction(recipient) { implicit request =>
            request.relay.disable(connection)
            Logger.info("Relay disabled: $recipient")
            returnUrl match {
              case "relaydetails" => Redirect(routes.RelayController.viewAliasRelay(connection,domainName,email,recipient))
              case _ => Redirect(routes.AliasController.viewAlias(connection,domainName,email))
            }
          }(connectionRequest)
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def disableRelay(connection: ConnectionName, recipient: String) = Authenticated.async { authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      RelayAction(recipient) { implicit request =>
        request.relay.disable(connection)
        Logger.info("Relay disabled: $recipient")
        Redirect(routes.AliasController.orphan(connection))
      }(connectionRequest)
    }(authRequest)
  }

   def enableAliasRelay(connection: ConnectionName, domainName: String, email: String, recipient: String, returnUrl: String) = Authenticated.async { authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        AliasAction(email).async { implicit aliasRequest =>
          RelayAction(recipient) { implicit request =>
            request.relay.enable(connection)
            Logger.info(s"Relay enabled: $recipient")
            returnUrl match {
              case "relaydetails" => Redirect(routes.RelayController.viewAliasRelay(connection,domainName,email,recipient))
              case _ => Redirect(routes.AliasController.viewAlias(connection,domainName,email))
            }
          }(connectionRequest)
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def enable(connection: ConnectionName, domainName: String, recipient: String, returnUrl: String) = Authenticated.async { authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName).async { implicit domainRequest =>
        RelayAction(recipient) { implicit request =>
          request.relay.enable(connection)
          Logger.info(s"Relay enabled: $recipient")
          returnUrl match {
            case "catchall" => Redirect(routes.AliasController.catchAll(connection))
            case "relaydetails" => Redirect(routes.RelayController.viewRelay(connection,domainName,recipient))
            case _ => Redirect(routes.DomainController.viewDomain(connection,domainName))
          }
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  val relayFormFields = mapping (
    "recipient" -> text,
    "enabled" -> ignored(false),
    "status" -> text
  )(Relay.apply)(Relay.unapply)

  val relayForm = Form( relayFormFields )


  def viewAdd(connection: ConnectionName, domainName: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName) { implicit domainRequest =>
        Ok(views.html.relay.addRelay( connection, domainRequest.domainRequested, relayForm, "domaindetails"))
      }(connectionRequest)
    }(authRequest)
  }

  def viewAddCatchAll(connection: ConnectionName, domainName: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName) { implicit domainRequest =>
        Ok(views.html.relay.addRelay( connection, domainRequest.domainRequested, relayForm.fill(Relay(s"@$domainName",false,"OK")), "catchall"))
      }(connectionRequest)
    }(authRequest)
  }

  def add(connection: ConnectionName, domainName: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName) { domainRequest =>
        relayForm.bindFromRequest()(domainRequest).fold(
          errors => {
            Logger.warn(s"Add relay form error")
            BadRequest(views.html.relay.addRelay( connection, domainRequest.domainRequested, errors, returnUrl ))
          },
          relay => {
            Relays.findRelay(connectionRequest.connection, relay.recipient) match {
              case None if FeatureToggles.isAddEnabled(connectionRequest.connection) => {
                relay.save(connection)
                returnUrl match {
                  case "catchall" => Redirect(routes.AliasController.catchAll(connection))
                  case "aliasdetails" => Redirect(routes.AliasController.catchAll(connection))
                  case _ => Redirect(routes.DomainController.viewDomain(connection,domainName))
                }
              }
              case None => {
                Logger.warn(s"Add feature not enabled")
                implicit val errorMessages = List(ErrorMessage("Add feature not enabled"))
                BadRequest(views.html.relay.addRelay( connection, domainRequest.domainRequested, relayForm.fill(relay), returnUrl))
              }
              case Some(_) => {
                Logger.warn(s"Relay ${relay.recipient} already exists")
                implicit val errorMessages = List(ErrorMessage("Relay already exist"))
                BadRequest(views.html.relay.addRelay( connection, domainRequest.domainRequested, relayForm.fill(relay), returnUrl))
              }
            }
          }
        )
      }(connectionRequest)
    }(authRequest)
  }

  def addAliasRelay(connection: ConnectionName, domainName: String, email: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { domainRequest =>
        AliasAction(email) { aliasRequest =>
          relayForm.bindFromRequest()(domainRequest).fold(
            errors => {
              Logger.warn(s"Add relay form error")
              implicit val errorMessages = List(ErrorMessage("Add relay failed"))
              val relays = Relays.findRelaysForAliasIfEnabled(connection, domainRequest.domainRequested, aliasRequest.alias)
              BadRequest(views.html.alias.aliasdetails( connection, domainRequest.domainRequested, aliasRequest.alias, relays ))
            },
            relay => {
              relay.save(connection)
              Redirect(routes.AliasController.viewAlias(connection,domainName,email))
            }
          )
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def remove(connection: ConnectionName, domainName: String, recipient: String, returnUrl: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName).async { implicit domainRequest =>
        RelayAction(recipient) { implicit request =>
          request.relay.delete(connection)
          Logger.info(s"Relay ${recipient} removed")
          returnUrl match {
            case "catchall" => Redirect(routes.AliasController.catchAll(connection))
            case "removedomain" => Redirect(routes.DomainController.viewRemove(connection,domainName))
            case _ => Redirect(routes.DomainController.viewDomain(connection,domainName))
          }
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def removeAliasRelay(connection: ConnectionName, domainName: String, email: String, recipient: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainAction(domainName).async { implicit domainRequest =>
        AliasAction(email).async { implicit aliasRequest =>
          RelayAction(recipient) { implicit request =>
            request.relay.delete(connection)
            Logger.info(s"Relay ${recipient} removed")
            Redirect(routes.AliasController.viewAlias(connection,domainName,email))
          }(connectionRequest)
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def removeRelay(connection: ConnectionName, recipient: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      RelayAction(recipient) { implicit request =>
        request.relay.delete(connection)
        Logger.info(s"Relay ${recipient} removed")
        Redirect(routes.AliasController.orphan(connection))
      }(connectionRequest)
    }(authRequest)
  }

  def viewRelay(connection: ConnectionName, domainName: String, recipient: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName).async { implicit domainRequest =>
        RelayAction(recipient) { implicit request =>
          val backup = Domains.findBackupDomain(connection,domainName)
          Ok(views.html.relay.relaydetails(connection,domainRequest.domainRequested,backup,None,request.relay))
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def viewAliasRelay(connection: ConnectionName, domainName: String, email: String, recipient: String) = Authenticated.async { implicit authRequest =>
    ConnectionAction(connection).async { implicit connectionRequest =>
      DomainOrBackupAction(domainName).async { implicit domainRequest =>
        AliasAction(email).async { implicit aliasRequest =>
          RelayAction(recipient) { implicit request =>
            Ok(views.html.relay.relaydetails(connection,domainRequest.domainRequested,None,Some(aliasRequest.alias),request.relay))
          }(connectionRequest)
        }(connectionRequest)
      }(connectionRequest)
    }(authRequest)
  }

  def rejectAliasRelay(connection: ConnectionName, domainName: String, email: String, recipient: String) = {
    Authenticated.async { implicit authRequest =>
      ConnectionAction(connection).async { implicit connectionRequest =>
        DomainOrBackupAction(domainName).async { implicit domainRequest =>
          AliasAction(email).async { implicit aliasRequest =>
            RelayAction(recipient) { implicit request =>
              request.relay.reject(connection)
              Logger.info(s"Relay ${recipient} set to reject")
              Redirect(routes.RelayController.viewAliasRelay(connection,domainName,email,recipient))
            }(connectionRequest)
          }(connectionRequest)
        }(connectionRequest)
      }(authRequest)
    }
  }

  def rejectRelay(connection: ConnectionName, domainName: String, recipient: String) = {
    Authenticated.async { implicit authRequest =>
      ConnectionAction(connection).async { implicit connectionRequest =>
        DomainOrBackupAction(domainName).async { implicit domainRequest =>
          RelayAction(recipient) { implicit request =>
            request.relay.reject(connection)
            Logger.info(s"Relay ${recipient} set to reject")
            Redirect(routes.RelayController.viewRelay(connection,domainName,recipient))
          }(connectionRequest)
        }(connectionRequest)
      }(authRequest)
    }
  }

  def acceptAliasRelay(connection: ConnectionName, domainName: String, email: String, recipient: String) = {
    Authenticated.async { implicit authRequest =>
      ConnectionAction(connection).async { implicit connectionRequest =>
        DomainOrBackupAction(domainName).async { implicit domainRequest =>
          AliasAction(email).async { implicit aliasRequest =>
            RelayAction(recipient) { implicit request =>
              request.relay.accept(connection)
              Logger.info(s"Relay ${recipient} set to accept")
              Redirect(routes.RelayController.viewAliasRelay(connection,domainName,email,recipient))
            }(connectionRequest)
          }(connectionRequest)
        }(connectionRequest)
      }(authRequest)
    }
  }

  def acceptRelay(connection: ConnectionName, domainName: String, recipient: String) = {
    Authenticated.async { implicit authRequest =>
      ConnectionAction(connection).async { implicit connectionRequest =>
        DomainOrBackupAction(domainName).async { implicit domainRequest =>
          RelayAction(recipient) { implicit request =>
            request.relay.accept(connection)
            Logger.info(s"Relay ${recipient} set to accept")
            Redirect(routes.RelayController.viewRelay(connection,domainName,recipient))
          }(connectionRequest)
        }(connectionRequest)
      }(authRequest)
    }
  }

}


