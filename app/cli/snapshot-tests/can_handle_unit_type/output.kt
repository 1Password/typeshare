package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// This struct has a unit field
@Serializable
data class StructHasVoidType (
	val thisIsAUnit: Unit
)

/// This enum has a variant associated with unit data
@Serializable
sealed class EnumHasVoidType {
	@Serializable
	@SerialName("hasAUnit")
	data class HasAUnit(val content: Unit): EnumHasVoidType()
}

