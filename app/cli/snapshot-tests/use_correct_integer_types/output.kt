package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

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

