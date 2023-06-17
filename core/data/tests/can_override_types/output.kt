@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

@Serializable
data class OverrideStruct (
	val fieldToOverride: Int
)

/// Generated type representing the anonymous struct variant `AnonymousStructVariant` of the `OverrideEnum` Rust enum
@Serializable
data class OverrideEnumAnonymousStructVariantInner (
	val fieldToOverride: Int
)

@Serializable
sealed class OverrideEnum {
	@Serializable
	@SerialName("UnitVariant")
	object UnitVariant: OverrideEnum()
	@Serializable
	@SerialName("TupleVariant")
	data class TupleVariant(val content: String): OverrideEnum()
	@Serializable
	@SerialName("AnonymousStructVariant")
	data class AnonymousStructVariant(val content: OverrideEnumAnonymousStructVariantInner): OverrideEnum()
}

