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

  def index = Action {
    Ok(views.html.domain.index())
  }

}


object AliasController extends Controller {

  def index = Action {
    Ok(views.html.alias.index())
  }

}


object UserController extends Controller {

  def index = Action {
    Ok(views.html.user.index())
  }

}
