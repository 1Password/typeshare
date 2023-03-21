@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

@Serializable
object ItemDetailsFieldValue

@Serializable
sealed class AdvancedColors {
	@Serializable
	@SerialName("string")
	data class String(val content: String): AdvancedColors()
	@Serializable
	@SerialName("number")
	data class Number(val content: Int): AdvancedColors()
	@Serializable
	@SerialName("number-array")
	data class NumberArray(val content: List<Int>): AdvancedColors()
	@Serializable
	@SerialName("reallyCoolType")
	data class ReallyCoolType(val content: ItemDetailsFieldValue): AdvancedColors()
}

