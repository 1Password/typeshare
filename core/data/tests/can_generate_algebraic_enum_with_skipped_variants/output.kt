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
	@SerialName("C")
	data class C(val content: Int): SomeEnum()
}

