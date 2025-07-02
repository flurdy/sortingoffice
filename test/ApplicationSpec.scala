import org.specs2.mutable._
import org.specs2.runner._
import org.junit.runner._

import play.api.test._
import play.api.test.Helpers._
import play.api.inject.guice.GuiceApplicationBuilder
import play.api.test.Helpers._

/**
 * Add your spec here.
 * You can mock out a whole application including requests, plugins etc.
 * For more information, consult the wiki.
 */
@RunWith(classOf[JUnitRunner])
class ApplicationSpec extends Specification {

  "Application" should {

    "send 404 on a bad request" in {
      val app = new GuiceApplicationBuilder().build()
      Helpers.running(app) {
        val request = FakeRequest(GET, "/boum")
        val result = route(app, request).get

        status(result) must equalTo(NOT_FOUND)
      }
    }

    "render the index page" in {
      val app = new GuiceApplicationBuilder().build()
      Helpers.running(app) {
        val request = FakeRequest(GET, "/")
        val result = route(app, request).get

        status(result) must equalTo(OK)
        contentType(result) must beSome.which(_ == "text/html")
        contentAsString(result) must contain ("Your new application is ready.")
      }
    }
  }
}
