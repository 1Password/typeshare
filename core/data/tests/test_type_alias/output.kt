@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

typealias Bar = String

@Serializable
data class Foo (
	val bar: Bar
)

