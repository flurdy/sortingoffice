package models

import com.github.t3hnar.bcrypt._
import play.api.Logger


case class LoginDetails(username: String, password: String)

case class RegisterDetails(username: String, password: String, confirmPassword: String)

case class ApplicationUser(username: String)


object ApplicationUsers {

	def authenticateLoginDetails(loginDetails: LoginDetails): Option[ApplicationUser] = {
		for{
			password <- Environment.findPasswordForApplicationUser(loginDetails.username)
			if matchPasswords(loginDetails,password)
		} yield ApplicationUser(loginDetails.username)
	}

	def matchPasswords(loginDetails: LoginDetails, password: String): Boolean = {
		loginDetails.password.isBcrypted(password) 
	}

	def register(registerDetails: RegisterDetails) {
		val encryptedPassword = registerDetails.password.bcrypt
		Logger.info(s"Password for ${registerDetails.username} is $encryptedPassword")
	} 

}
