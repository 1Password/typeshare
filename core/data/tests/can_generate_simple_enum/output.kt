@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

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

