package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// A struct with no target_os. Should be generated when
/// we use --target-os.
@Serializable
object AlwaysAccept

@Serializable
data class DefinedTwice (
	val field1: String
)

@Serializable
object Excluded

@Serializable
object ManyStruct

@Serializable
object MultipleTargets

@Serializable
object NestedNotTarget1

@Serializable
object OtherExcluded

@Serializable
enum class AlwaysAcceptEnum(val string: String) {
	@SerialName("Variant1")
	Variant1("Variant1"),
	@SerialName("Variant2")
	Variant2("Variant2"),
}

@Serializable
enum class SomeEnum(val string: String) {
}

/// Generated type representing the anonymous struct variant `Variant7` of the `TestEnum` Rust enum
@Serializable
data class TestEnumVariant7Inner (
	val field1: String
)

/// Generated type representing the anonymous struct variant `Variant9` of the `TestEnum` Rust enum
@Serializable
data class TestEnumVariant9Inner (
	val field2: String
)

@Serializable
sealed class TestEnum {
	@Serializable
	@SerialName("Variant1")
	object Variant1: TestEnum()
	@Serializable
	@SerialName("Variant5")
	object Variant5: TestEnum()
	@Serializable
	@SerialName("Variant7")
	data class Variant7(val content: TestEnumVariant7Inner): TestEnum()
	@Serializable
	@SerialName("Variant8")
	object Variant8: TestEnum()
	@Serializable
	@SerialName("Variant9")
	data class Variant9(val content: TestEnumVariant9Inner): TestEnum()
}

