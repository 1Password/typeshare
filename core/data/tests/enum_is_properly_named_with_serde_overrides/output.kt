@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

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

