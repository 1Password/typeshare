@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

/// This is a comment.
@Serializable
data class Foo (
	val a: Byte,
	val b: Short,
	val c: Int,
	val e: UByte,
	val f: UShort,
	val g: UInt
)

