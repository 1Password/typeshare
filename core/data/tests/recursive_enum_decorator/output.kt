@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

@Serializable
sealed class Options {
	@Serializable
	@SerialName("red")
	data class Red(val content: Boolean): Options()
	@Serializable
	@SerialName("banana")
	data class Banana(val content: String): Options()
	@Serializable
	@SerialName("vermont")
	data class Vermont(val content: Options): Options()
}

/// Generated type representing the anonymous struct variant `Exactly` of the `MoreOptions` Rust enum
@Serializable
data class MoreOptionsExactlyInner (
	val config: String
)

/// Generated type representing the anonymous struct variant `Built` of the `MoreOptions` Rust enum
@Serializable
data class MoreOptionsBuiltInner (
	val top: MoreOptions
)

@Serializable
sealed class MoreOptions {
	@Serializable
	@SerialName("news")
	data class News(val content: Boolean): MoreOptions()
	@Serializable
	@SerialName("exactly")
	data class Exactly(val content: MoreOptionsExactlyInner): MoreOptions()
	@Serializable
	@SerialName("built")
	data class Built(val content: MoreOptionsBuiltInner): MoreOptions()
}

