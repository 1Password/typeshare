package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
object MultipleTargets

@Serializable
enum class SomeEnum(val string: String) {
}

/// Generated type representing the anonymous struct variant `Variant7` of the `TestEnum` Rust enum
@Serializable
object TestEnumVariant7Inner

@Serializable
sealed class TestEnum {
	@Serializable
	@SerialName("Variant5")
	object Variant5: TestEnum()
	@Serializable
	@SerialName("Variant7")
	data class Variant7(val content: TestEnumVariant7Inner): TestEnum()
	@Serializable
	@SerialName("Variant8")
	object Variant8: TestEnum()
}

