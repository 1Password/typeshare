@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

@Serializable
sealed class SomeEnum {
	@Serializable
	@SerialName("A")
	object A: SomeEnum()
	@Serializable
	@SerialName("B")
	data class B(val field1: String): SomeEnum()
	@Serializable
	@SerialName("C")
	data class C(val field1: UInt, val field2: Float): SomeEnum()
	@Serializable
	@SerialName("D")
	data class D(val field3: Boolean?): SomeEnum()
}

