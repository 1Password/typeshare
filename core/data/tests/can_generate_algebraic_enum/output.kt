@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

/// Struct comment
@Serializable
object ItemDetailsFieldValue

/// Enum comment
@Serializable
sealed class AdvancedColors {
	/// This is a case comment
	@Serializable
	@SerialName("String")
	data class String(val content: String): AdvancedColors()
	@Serializable
	@SerialName("Number")
	data class Number(val content: Int): AdvancedColors()
	@Serializable
	@SerialName("UnsignedNumber")
	data class UnsignedNumber(val content: UInt): AdvancedColors()
	@Serializable
	@SerialName("NumberArray")
	data class NumberArray(val content: List<Int>): AdvancedColors()
	/// Comment on the last element
	@Serializable
	@SerialName("ReallyCoolType")
	data class ReallyCoolType(val content: ItemDetailsFieldValue): AdvancedColors()
}

@Serializable
sealed class AdvancedColors2 {
	/// This is a case comment
	@Serializable
	@SerialName("string")
	data class String(val content: String): AdvancedColors2()
	@Serializable
	@SerialName("number")
	data class Number(val content: Int): AdvancedColors2()
	@Serializable
	@SerialName("number-array")
	data class NumberArray(val content: List<Int>): AdvancedColors2()
	/// Comment on the last element
	@Serializable
	@SerialName("really-cool-type")
	data class ReallyCoolType(val content: ItemDetailsFieldValue): AdvancedColors2()
}

