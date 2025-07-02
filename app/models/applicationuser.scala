package models

import com.github.t3hnar.bcrypt._
import org.slf4j.LoggerFactory


case class LoginDetails(username: String, password: String)

case class RegisterDetails(username: String, password: String, confirmPassword: String)

case class ApplicationUser(username: String)

class ApplicationUsers(environment: Environment) {
	private val logger = LoggerFactory.getLogger(getClass)

	def authenticateLoginDetails(loginDetails: LoginDetails): Option[ApplicationUser] = {
		for{
			password <- environment.findPasswordForApplicationUser(loginDetails.username)
			if matchPasswords(loginDetails,password)
		} yield ApplicationUser(loginDetails.username)
	}

	def matchPasswords(loginDetails: LoginDetails, password: String): Boolean = {
		loginDetails.password.isBcrypted(password)
	}

	def register(registerDetails: RegisterDetails): Unit = {
		val encryptedPassword = registerDetails.password.bcrypt
		logger.info(s"Password for ${registerDetails.username} is $encryptedPassword")
	}

}
