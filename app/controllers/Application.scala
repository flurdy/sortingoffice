package controllers

import play.api._
import play.api.mvc._
import models._

object Application extends Controller {

  def index = Action {
    Ok(views.html.index())
  }

  def about = Action {
    Ok(views.html.about())
  }

  def contact = Action {
    Ok(views.html.contact())
  }

}


object DomainController extends Controller {

  def domain = Action {
    val relays = Domains.findRelayDomains
    val backups = Domains.findBackupDomains
    Ok(views.html.domain.domain( relays, backups ))
  }

  def alias = Action {
    val domain = Domain("",true,"")
    val relays = RelayRepository.findRelaysForDomain(domain)
    val aliases = AliasRepository.findAliasesForDomain(domain)
    val users = UserRepository.findUsersForDomain(domain)
    Ok(views.html.domain.domainalias(relays,aliases,users))
  }

}


object AliasController extends Controller {

  def alias = Action {
    Ok(views.html.alias.alias())
  }

  def catchAll = Action {
    Ok(views.html.alias.catchall())
  }

  def common = Action {
    Ok(views.html.alias.common())
  }

  def crossDomain = Action {
    Ok(views.html.alias.cross())
  }

}


object UserController extends Controller {

  def user = Action {
    Ok(views.html.user.user())
  }
  def edituser = Action {
    Ok(views.html.user.edituser())
  }

}
