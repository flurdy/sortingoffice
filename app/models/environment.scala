package models

import scala.jdk.CollectionConverters._
import play.api.Configuration
import com.typesafe.config.Config

class Environment(config: Configuration) {
	import Environment.ConnectionName

	val connectionNames: List[ConnectionName] =
		config.getOptional[Seq[String]]("databases.connections").map(_.toList).getOrElse(List.empty)

	def connectionDescription(connection: ConnectionName): Option[String] =
		config.getOptional[String](s"databases.${connection}.description")

	val databaseConnections: List[(ConnectionName, String)] =
		connectionNames.map(name => (name, connectionDescription(name).getOrElse("")))

	def findPasswordForApplicationUser(username: String): Option[String] = {
		for {
			userConfigs <- config.getOptional[Configuration]("application.users")
			userConfig  <- userConfigs.getOptional[Configuration](username)
			password    <- userConfig.getOptional[String]("password")
		} yield password
	}
}

object Environment {
	type ConnectionName = String
}

case class FeatureToggle(name: String, enabled: Boolean)

case class FeatureToggleMap(toggles: Map[String, FeatureToggle]) {
	def isEnabled(featureName: String) = toggles.get(featureName).map(_.enabled).getOrElse(false)
}

class FeatureToggles(config: Configuration) {
	private val featureNames = List("toggle", "add", "remove", "edit")

	private def isDatabaseFeatureEnabled(connection: String, featureName: String): Boolean =
		config.getOptional[Boolean](s"databases.${connection}.features.${featureName}").getOrElse(false)

	def findFeatureToggles(connection: String): FeatureToggleMap = {
		val enabledFeatures = featureNames.filter(isDatabaseFeatureEnabled(connection, _))
		val map = enabledFeatures.map(feature => (feature, FeatureToggle(feature, true))).toMap
		FeatureToggleMap(map)
	}

	def isBackupEnabled(connection: String): Boolean = isDatabaseFeatureEnabled(connection, "backup")
	def isRelayEnabled(connection: String): Boolean = isDatabaseFeatureEnabled(connection, "relay")
	def isRelocationEnabled(connection: String): Boolean = isDatabaseFeatureEnabled(connection, "relocation")
	def isToggleEnabled(connection: String): Boolean = isDatabaseFeatureEnabled(connection, "toggle")
	def isAddEnabled(connection: String): Boolean = isDatabaseFeatureEnabled(connection, "add")
	def isRemoveEnabled(connection: String): Boolean = isDatabaseFeatureEnabled(connection, "remove")
	def isEditEnabled(connection: String): Boolean = isDatabaseFeatureEnabled(connection, "edit")
}

case class ErrorMessage(message: String)
