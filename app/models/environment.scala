package models

import play.api.Play
import play.api.Play.current
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


