@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

@Serializable
data class A (
	val field: UInt
)

@Serializable
data class ABC (
	val field: UInt
)

@Serializable
data class AB (
	val field: UInt
)

@Serializable
data class OutsideOfModules (
	val field: UInt
)

