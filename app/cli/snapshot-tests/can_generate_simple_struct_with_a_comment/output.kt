package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
object Location

/// This is a comment.
@Serializable
data class Person (
	/// This is another comment
	val name: String,
	val age: UByte,
	val info: String? = null,
	val emails: List<String>,
	val location: Location
)

