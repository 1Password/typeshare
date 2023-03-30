@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

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
data class D (
	val dependsOn: C,
	val alsoDependsOn: E? = null
)

@Serializable
data class E (
	val dependsOn: D
)

