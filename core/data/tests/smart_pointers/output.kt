package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// This is a comment.
@Serializable
data class ArcyColors (
	val red: UByte,
	val blue: String,
	val green: List<String>
)

/// This is a comment.
@Serializable
data class CellyColors (
	val red: String,
	val blue: List<String>
)

/// This is a comment.
@Serializable
data class CowyColors (
	val lifetime: String
)

/// This is a comment.
@Serializable
data class LockyColors (
	val red: String
)

/// This is a comment.
@Serializable
data class MutexyColors (
	val blue: List<String>,
	val green: String
)

/// This is a comment.
@Serializable
data class RcyColors (
	val red: String,
	val blue: List<String>,
	val green: String
)

/// This is a comment.
@Serializable
sealed class BoxyColors {
	@Serializable
	@SerialName("Red")
	object Red: BoxyColors()
	@Serializable
	@SerialName("Blue")
	object Blue: BoxyColors()
	@Serializable
	@SerialName("Green")
	data class Green(val content: String): BoxyColors()
}

