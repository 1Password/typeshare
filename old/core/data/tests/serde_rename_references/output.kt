package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

typealias AliasTest = List<SomethingFoo>

@Serializable
data class Test (
	val field1: SomethingFoo,
	val field2: SomethingFoo? = null
)

@Serializable
enum class SomethingFoo(val string: String) {
	@SerialName("A")
	A("A"),
}

@Serializable
sealed class Parent {
	@Serializable
	@SerialName("B")
	data class B(val value: SomethingFoo): Parent()
}

