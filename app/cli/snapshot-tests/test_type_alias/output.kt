package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

typealias Bar = String

@Serializable
data class Foo (
	val bar: Bar
)

