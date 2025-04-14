package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
object OtherType

/// This is a comment.
@Serializable
data class PersonTwo (
	val name: String,
	val age: UByte,
	val extraSpecialFieldOne: Int,
	val extraSpecialFieldTwo: List<String>? = null,
	val nonStandardDataType: OtherType,
	val nonStandardDataTypeInArray: List<OtherType>? = null
)

