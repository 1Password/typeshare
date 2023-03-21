@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

/// This is a comment.
@Serializable
enum class Colors(val string: String) {
	@SerialName("Green\"")
	Green("Green\""),
}

