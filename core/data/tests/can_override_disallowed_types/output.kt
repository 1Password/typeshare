package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class DisallowedType (
	val disallowed_type: ULong,
	val another_disallowed_type: Long,
	val disallowed_type_serde_with: ULong
)

