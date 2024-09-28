package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class A (
	val field: UInt
)

@Serializable
data class B (
	val dependsOn: A
)

@Serializable
data class C (
	val dependsOn: B
)

@Serializable
data class E (
	val dependsOn: D
)

@Serializable
data class D (
	val dependsOn: C,
	val alsoDependsOn: E? = null
)

