package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
@JvmInline
value class BestHockeyTeams5(
	val value: String
) {
	override fun toString(): String = "***"
}

@Serializable
data class BestHockeyTeams (
	val PittsburghPenguins: UInt,
	val Lies: String
)

@Serializable
data class BestHockeyTeams1 (
	val PittsburghPenguins: UInt,
	val Lies: String
)

@Serializable
data class BestHockeyTeams2 (
	val PittsburghPenguins: UInt,
	val Lies: String
)

@Serializable
data class BestHockeyTeams3 (
	val PittsburghPenguins: UInt,
	val Lies: String
) {
	override fun toString(): String = "BestHockeyTeams3"
}

@Serializable
data class BestHockeyTeams4 (
	val PittsburghPenguins: UInt,
	val Lies: String
)

