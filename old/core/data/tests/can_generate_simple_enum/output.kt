package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// This is a comment.
/// Continued lovingly here
@Serializable
enum class Colors(val string: String) {
	@SerialName("Red")
	Red("Red"),
	@SerialName("Blue")
	Blue("Blue"),
	/// Green is a cool color
	@SerialName("Green")
	Green("Green"),
}

