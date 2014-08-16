package models

import play.api.Play
import play.api.Play.current
import play.api.Logger
import scala.collection.JavaConverters._


object Environment {

	type ConnectionName = String

	val connectionNames: List[ConnectionName] = {
		Play.configuration.getStringList("databases.connections").map(_.asScala.toList).getOrElse(List.empty)
	}

	def connectionDescription(connection: ConnectionName): Option[String] = {
		Play.configuration.getString(s"databases.${connection}.description")
	}

	val databaseConnections: List[(ConnectionName,String)] = {
		connectionNames.map( name => (name, connectionDescription(name).getOrElse("")))
	}

}


import Environment.ConnectionName

object FeatureToggles {

	private def isFeatureEnabled(featureName: String): Boolean = {
		Play.configuration.getBoolean(s"feature.${featureName}.enabled").getOrElse(false)
	}

	private def isDatabaseFeatureEnabled(connection: ConnectionName, featureName: String): Boolean = {
		Play.configuration.getBoolean(s"databases.${connection}.features.${featureName}").getOrElse(false)
	}

	def isBackupEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"backup")

	def isRelayEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"relay")

	def isRelocationEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"relocation")

}


case class ErrorMessage(message: String)


