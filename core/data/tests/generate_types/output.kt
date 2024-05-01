package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
object CustomType

@Serializable
data class Types (
	val s: String,
	val static_s: String,
	val int8: Byte,
	val float: Float,
	val double: Double,
	val array: List<String>,
	val fixed_length_array: List<String>,
	val dictionary: HashMap<String, Int>,
	val optional_dictionary: HashMap<String, Int>? = null,
	val custom_type: CustomType
)

