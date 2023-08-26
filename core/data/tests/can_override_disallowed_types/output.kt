@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

@Serializable
data class DisallowedType (
	val disallowed_type: ULong,
	val another_disallowed_type: Long,
	val disallowed_type_serde_with: ULong
)

