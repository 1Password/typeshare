package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
enum class TestEnum(val string: String) {
	@SerialName("Variant1")
	Variant1("Variant1"),
}

