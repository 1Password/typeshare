package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// Generated type representing the anonymous struct variant `Us` of the `AutofilledBy` Rust enum
@Serializable
data class AutofilledByUsInner (
	/// The UUID for the fill
	val uuid: String
)

/// Generated type representing the anonymous struct variant `SomethingElse` of the `AutofilledBy` Rust enum
@Serializable
data class AutofilledBySomethingElseInner (
	/// The UUID for the fill
	val uuid: String
)

/// Enum keeping track of who autofilled a field
@Serializable
sealed class AutofilledBy {
	/// This field was autofilled by us
	@Serializable
	@SerialName("Us")
	data class Us(val content: AutofilledByUsInner): AutofilledBy()
	/// Something else autofilled this field
	@Serializable
	@SerialName("SomethingElse")
	data class SomethingElse(val content: AutofilledBySomethingElseInner): AutofilledBy()
}

