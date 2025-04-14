package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// This is a comment.
@Serializable
enum class Colors(val string: String) {
	@SerialName("red")
	Red("red"),
	@SerialName("blue-ish")
	Blue("blue-ish"),
	@SerialName("Green")
	Green("Green"),
}

