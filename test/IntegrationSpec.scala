import org.specs2.mutable._
import org.specs2.runner._
import org.junit.runner._

import play.api.test._
import play.api.test.Helpers._
import play.api.inject.guice.GuiceApplicationBuilder
import play.api.test.Helpers._

/**
 * add your integration spec here.
 * An integration test will fire up a whole play application in a real (or headless) browser
 */
@RunWith(classOf[JUnitRunner])
class IntegrationSpec extends Specification {

  "Application" should {

    "work from within a browser" in {
      val app = new GuiceApplicationBuilder().build()
      Helpers.running(app) {
        val request = FakeRequest(GET, "/")
        val result = route(app, request).get

        status(result) must equalTo(OK)
        contentAsString(result) must contain("Your new application is ready.")
      }
    }
  }
}
