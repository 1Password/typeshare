package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// This is a comment.
/// Continued lovingly here
@Serializable
enum class Colors(val string: String) {
	@SerialName("red")
	Red("red"),
	@SerialName("blue")
	Blue("blue"),
	/// Green is a cool color
	@SerialName("green-like")
	Green("green-like"),
}

