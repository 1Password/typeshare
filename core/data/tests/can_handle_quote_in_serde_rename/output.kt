package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// This is a comment.
@Serializable
enum class Colors(val string: String) {
	@SerialName("Green\"")
	Green("Green\""),
}

