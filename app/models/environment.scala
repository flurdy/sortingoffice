package models

import play.api.Play
import play.api.Play.current
import play.api.Logger
import scala.collection.JavaConverters._
import com.typesafe.config.Config


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

	def findPasswordForApplicationUser(username: String): Option[String] = {
		for{
			 userConfigs <- Play.configuration.getConfig("application.users")	
			 userConfig  <- userConfigs.getConfig(username)
			 password    <- userConfig.getString("password")
		} yield password	
	}

}


import Environment.ConnectionName


case class FeatureToggle(name: String, enabled: Boolean)

case class FeatureToggleMap(toggles: Map[String,FeatureToggle]){
	def isEnabled(featureName: String) = toggles.get(featureName).map(_.enabled).getOrElse(false)
}

object FeatureToggles {

	private val featureNames = List("toggle","add","remove","edit")

	private def isDatabaseFeatureEnabled(connection: ConnectionName, featureName: String): Boolean = {
		Play.configuration.getBoolean(s"databases.${connection}.features.${featureName}").getOrElse(false)
	}

	def findFeatureToggles(connection: ConnectionName): FeatureToggleMap = {
		val enabledFeatures = featureNames.filter( isDatabaseFeatureEnabled(connection,_))
		val map = enabledFeatures.map( feature => (feature,FeatureToggle(feature,true)) ).toMap
		FeatureToggleMap(map)
	}

	def isBackupEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"backup")

	def isRelayEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"relay")

	def isRelocationEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"relocation")

	def isToggleEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"toggle")

	def isAddEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"add")

	def isRemoveEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"remove")

	def isEditEnabled(connection: ConnectionName): Boolean = isDatabaseFeatureEnabled(connection,"edit")

}


case class ErrorMessage(message: String)


