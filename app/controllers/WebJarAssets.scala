package controllers

import javax.inject._
import play.api.mvc._
import play.api.Environment
import play.api.http.HttpErrorHandler
import play.api.routing.Router
import play.api.mvc.PathBindable
import play.api.mvc.Results.NotFound
import play.api.libs.Files.SingletonTemporaryFileCreator
import play.api.libs.Files.TemporaryFileCreator
import play.api.Environment
import play.api.http.HttpEntity
import play.api.http.MimeTypes
import play.api.libs.Files
import play.api.mvc._
import scala.concurrent.ExecutionContext
import play.api.Logging

@Singleton
class WebJarAssets @Inject()(
  cc: ControllerComponents
) extends AbstractController(cc) with Logging {

  private val webjarsPrefix = "/META-INF/resources/webjars/"

  def at(file: String): Action[AnyContent] = Action { implicit request =>
    val resourcePath = s"$webjarsPrefix$file"
    val resource = Option(getClass.getResourceAsStream(resourcePath))
    resource match {
      case Some(stream) =>
        val bytes = stream.readAllBytes()
        stream.close()
        val contentType = MimeTypes.forFileName(file).getOrElse("application/octet-stream")
        Ok(bytes).as(contentType)
      case None =>
        logger.warn(s"WebJar asset not found: $file")
        NotFound(s"WebJar asset not found: $file")
    }
  }
}
