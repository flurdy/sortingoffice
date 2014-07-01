package controllers

import play.api._
import play.api.mvc._

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
    Ok(views.html.domain.domain())
  }

  def alias = Action {
    Ok(views.html.domain.domainalias())
  }

}


object AliasController extends Controller {

  def alias = Action {
    Ok(views.html.alias.alias())
  }

}


object UserController extends Controller {

  def user = Action {
    Ok(views.html.user.user())
  }

}
