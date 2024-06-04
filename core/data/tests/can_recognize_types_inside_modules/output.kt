package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class A (
	val field: UInt
)

@Serializable
data class AB (
	val field: UInt
)

@Serializable
data class ABC (
	val field: UInt
)

@Serializable
data class OutsideOfModules (
	val field: UInt
)

