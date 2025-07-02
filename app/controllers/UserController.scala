package controllers

import javax.inject._
import scala.concurrent.{ExecutionContext, Future}
import play.api.mvc._
import play.api.data._
import play.api.data.Forms._
import play.api.Logging
import models._
import infrastructure._

@Singleton
class UserController @Inject()(
  cc: MessagesControllerComponents,
  userRepository: UserRepository,
  featureToggles: FeatureToggles,
  domains: Domains,
  aliases: Aliases,
  users: Users,
  aliasRepository: AliasRepository
)(implicit ec: ExecutionContext)
  extends AbstractController(cc)
  with Logging {

  // --- Forms ---
  val userFormFields = mapping(
    "email" -> text,
    "name" -> text,
    "maildir" -> text,
    "passwordReset" -> ignored(true),
    "enabled" -> ignored(false)
  )(User.apply)(User.unapply)
  val userForm = Form(userFormFields)

  val updateUserFormFields = mapping(
    "email" -> ignored("invalid@example"),
    "name" -> text,
    "maildir" -> text,
    "passwordReset" -> ignored(true),
    "enabled" -> ignored(false)
  )(User.apply)(User.unapply)
  val updateUserForm = Form(updateUserFormFields)

  // --- Helper: UserAction ---
  private def withUser(connection: String, email: String)(block: User => Future[Result]): Future[Result] = {
    val togglesMap = featureToggles.findFeatureToggles(connection)
    userRepository.findUser(connection, email) match {
      case Some(user) =>
        logger.debug(s"User $email found")
        block(user)
      case None =>
        logger.warn(s"User $email not found")
        val users = userRepository.findUsers(connection)
        val errorMessages = List(ErrorMessage("User not found"))
        Future.successful(NotFound(views.html.user.user(connection, users)(errorMessages, togglesMap, None)))
    }
  }

  // Helper: get feature toggles for a connection (for template rendering only)
  private def togglesMap(connection: String): FeatureToggleMap = featureToggles.findFeatureToggles(connection)

  // --- Actions ---
  def user(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggleMap: FeatureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    val users = userRepository.findUsers(connection)
    Ok(views.html.user.user(connection, users))
  }

  def viewUser(connection: String, email: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggleMap: FeatureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    withUser(connection, email) { user =>
      val domain = user.findDomain(connection, domains, aliases)
      val alias = user.findAlias(connection, aliasRepository)
      Future.successful(Ok(views.html.user.edituser(connection, user, domain, alias, updateUserForm)))
    }
  }

  def disableUser(connection: String, email: String, returnUrl: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    withUser(connection, email) { user =>
      user.disable(connection, featureToggles, userRepository)
      logger.info(s"User disabled: $email")
      val redirect = returnUrl match {
        case "orphan" => Redirect(routes.AliasController.orphan(connection))
        case "edituser" => Redirect(routes.UserController.viewUser(connection, email))
        case _ => Redirect(routes.UserController.user(connection))
      }
      Future.successful(redirect)
    }
  }

  def enableUser(connection: String, email: String, returnUrl: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    withUser(connection, email) { user =>
      user.enable(connection, featureToggles, userRepository)
      logger.info(s"User enabled: $email")
      val redirect = returnUrl match {
        case "orphan" => Redirect(routes.AliasController.orphan(connection))
        case "edituser" => Redirect(routes.UserController.viewUser(connection, email))
        case _ => Redirect(routes.UserController.user(connection))
      }
      Future.successful(redirect)
    }
  }

  def disable(connection: String, domain: String, email: String, returnUrl: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    withUser(connection, email) { user =>
      user.disable(connection, featureToggles, userRepository)
      logger.info(s"User disabled: $email")
      val redirect = returnUrl match {
        case "orphan" => Redirect(routes.AliasController.orphan(connection))
        case "edituser" => Redirect(routes.UserController.viewUser(connection, email))
        case _ => Redirect(routes.DomainController.viewDomain(connection, domain))
      }
      Future.successful(redirect)
    }
  }

  def enable(connection: String, domain: String, email: String, returnUrl: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    withUser(connection, email) { user =>
      user.enable(connection, featureToggles, userRepository)
      logger.info(s"User enabled: $email")
      val redirect = returnUrl match {
        case "orphan" => Redirect(routes.AliasController.orphan(connection))
        case "edituser" => Redirect(routes.UserController.viewUser(connection, email))
        case _ => Redirect(routes.DomainController.viewDomain(connection, domain))
      }
      Future.successful(redirect)
    }
  }

  def viewAdd(connection: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggleMap: FeatureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    Ok(views.html.user.addUser(connection, None, userForm))
  }

  def viewAddWithDomain(connection: String, domainName: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureToggleMap: FeatureToggleMap = featureToggles.findFeatureToggles(connection)
    implicit val currentUser: Option[ApplicationUser] = None
    val domainOpt = domains.findDomain(connection, domainName)
    Ok(views.html.user.addUser(connection, domainOpt, userForm))
  }

  def add(connection: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    val togglesMap = featureToggles.findFeatureToggles(connection)
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = togglesMap
    implicit val currentUser: Option[ApplicationUser] = None
    userForm.bindFromRequest().fold(
      errors => {
        logger.warn(s"Add user form error")
        Future.successful(BadRequest(views.html.user.addUser(connection, None, errors)))
      },
      user => {
        userRepository.findUser(connection, user.email) match {
          case None =>
            userRepository.findUserByMaildir(connection, user.maildir) match {
              case None if featureToggles.isAddEnabled(connection) =>
                user.save(connection, featureToggles, userRepository)
                logger.info(s"User ${user.email} added")
                Future.successful(Redirect(routes.UserController.user(connection)))
              case None =>
                logger.warn(s"Add feature not enabled")
                val errorMessages = List(ErrorMessage("Add feature not enabled"))
                Future.successful(BadRequest(views.html.user.addUser(connection, None, userForm.fill(user))(errorMessages, togglesMap, None)))
              case Some(_) =>
                logger.warn(s"User maildir ${user.maildir} already exists")
                val errorMessages = List(ErrorMessage("User's maildir already exist"))
                Future.successful(BadRequest(views.html.user.addUser(connection, None, userForm.fill(user))(errorMessages, togglesMap, None)))
            }
          case Some(_) =>
            logger.warn(s"User ${user.email} already exists")
            val errorMessages = List(ErrorMessage("User already exist"))
            Future.successful(BadRequest(views.html.user.addUser(connection, None, userForm.fill(user))(errorMessages, togglesMap, None)))
        }
      }
    )
  }

  def addWithDomain(connection: String, domainName: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    val togglesMap = featureToggles.findFeatureToggles(connection)
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = togglesMap
    implicit val currentUser: Option[ApplicationUser] = None
    val domainOpt = domains.findDomain(connection, domainName)
    userForm.bindFromRequest().fold(
      errors => {
        logger.warn(s"Add user form error")
        Future.successful(BadRequest(views.html.user.addUser(connection, domainOpt, errors)))
      },
      user => {
        userRepository.findUser(connection, user.email) match {
          case None if featureToggles.isAddEnabled(connection) =>
            user.save(connection, featureToggles, userRepository)
            logger.info(s"User ${user.email} added")
            Future.successful(Redirect(routes.DomainController.viewDomain(connection, domainName)))
          case None =>
            logger.warn(s"Add feature not enabled")
            val errorMessages = List(ErrorMessage("Add feature not enabled"))
            Future.successful(BadRequest(views.html.user.addUser(connection, domainOpt, userForm.fill(user))(errorMessages, togglesMap, None)))
          case Some(_) =>
            logger.warn(s"User ${user.email} already exists")
            val errorMessages = List(ErrorMessage("User already exist"))
            Future.successful(BadRequest(views.html.user.addUser(connection, domainOpt, userForm.fill(user))(errorMessages, togglesMap, None)))
        }
      }
    )
  }

  def remove(connection: String, email: String, returnUrl: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    withUser(connection, email) { user =>
      user.delete(connection, featureToggles, userRepository)
      logger.info(s"User removed: $email")
      val redirect = returnUrl match {
        case "orphan" => Redirect(routes.AliasController.orphan(connection))
        case _ => Redirect(routes.UserController.user(connection))
      }
      Future.successful(redirect)
    }
  }

  def removeDomainUser(connection: String, domainName: String, email: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    withUser(connection, email) { user =>
      user.delete(connection, featureToggles, userRepository)
      logger.info(s"User removed: $email")
      Future.successful(Redirect(routes.DomainController.viewRemove(connection, domainName)))
    }
  }

  def resetPassword(connection: String, email: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    withUser(connection, email) { user =>
      user.resetPassword(connection, featureToggles, userRepository)
      logger.info(s"User password reset: $email")
      Future.successful(Redirect(routes.UserController.viewUser(connection, email)))
    }
  }

  def update(connection: String, email: String): Action[AnyContent] = Action.async { implicit request: Request[AnyContent] =>
    withUser(connection, email) { user =>
      updateUserForm.bindFromRequest().fold(
        errors => {
          logger.warn(s"Update user form error")
          val domain = user.findDomain(connection, domains, aliases)
          val alias = user.findAlias(connection, aliasRepository)
          implicit val featureToggleMap: FeatureToggleMap = featureToggles.findFeatureToggles(connection)
          Future.successful(BadRequest(views.html.user.edituser(connection, user, domain, alias, errors)))
        },
        updatedUser => {
          user.copy(name = updatedUser.name, maildir = updatedUser.maildir).update(connection, featureToggles, userRepository)
          logger.info(s"User updated: $email")
          Future.successful(Redirect(routes.UserController.viewUser(connection, email)))
        }
      )
    }
  }

  def viewAdd(connection: String, domainName: String): Action[AnyContent] = Action { implicit request: Request[AnyContent] =>
    val togglesMap = featureToggles.findFeatureToggles(connection)
    implicit val errorMessages: List[ErrorMessage] = List.empty
    implicit val featureTogglesMap: FeatureToggleMap = togglesMap
    implicit val currentUser: Option[ApplicationUser] = None
    val domainOpt = domains.findDomain(connection, domainName)
    Ok(views.html.user.addUser(connection, domainOpt, userForm))
  }
}
