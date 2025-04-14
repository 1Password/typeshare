package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

/// Generated type representing the anonymous struct variant `Us` of the `AutofilledBy` Rust enum
@Serializable
data class AutofilledByUsInner (
	/// The UUID for the fill
	val uuid: String
)

/// Generated type representing the anonymous struct variant `SomethingElse` of the `AutofilledBy` Rust enum
@Serializable
data class AutofilledBySomethingElseInner (
	/// The UUID for the fill
	val uuid: String,
	/// Some other thing
	val thing: Int
)

/// Enum keeping track of who autofilled a field
@Serializable
sealed class AutofilledBy {
	/// This field was autofilled by us
	@Serializable
	@SerialName("Us")
	data class Us(val content: AutofilledByUsInner): AutofilledBy()
	/// Something else autofilled this field
	@Serializable
	@SerialName("SomethingElse")
	data class SomethingElse(val content: AutofilledBySomethingElseInner): AutofilledBy()
}

/// Generated type representing the anonymous struct variant `AnonVariant` of the `EnumWithManyVariants` Rust enum
@Serializable
data class EnumWithManyVariantsAnonVariantInner (
	val uuid: String
)

/// Generated type representing the anonymous struct variant `AnotherAnonVariant` of the `EnumWithManyVariants` Rust enum
@Serializable
data class EnumWithManyVariantsAnotherAnonVariantInner (
	val uuid: String,
	val thing: Int
)

/// This is a comment (yareek sameek wuz here)
@Serializable
sealed class EnumWithManyVariants {
	@Serializable
	@SerialName("UnitVariant")
	object UnitVariant: EnumWithManyVariants()
	@Serializable
	@SerialName("TupleVariantString")
	data class TupleVariantString(val content: String): EnumWithManyVariants()
	@Serializable
	@SerialName("AnonVariant")
	data class AnonVariant(val content: EnumWithManyVariantsAnonVariantInner): EnumWithManyVariants()
	@Serializable
	@SerialName("TupleVariantInt")
	data class TupleVariantInt(val content: Int): EnumWithManyVariants()
	@Serializable
	@SerialName("AnotherUnitVariant")
	object AnotherUnitVariant: EnumWithManyVariants()
	@Serializable
	@SerialName("AnotherAnonVariant")
	data class AnotherAnonVariant(val content: EnumWithManyVariantsAnotherAnonVariantInner): EnumWithManyVariants()
}

