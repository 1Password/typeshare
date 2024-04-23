package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

typealias OptionalU32 = UInt?

typealias OptionalU16 = UShort?

@Serializable
data class FooBar (
	val foo: OptionalU32,
	val bar: OptionalU16
)

