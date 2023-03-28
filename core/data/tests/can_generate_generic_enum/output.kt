@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

@Serializable
sealed class GenericEnum<A, B> {
	@Serializable
	@SerialName("VariantA")
	data class VariantA<A, B>(val content: A): GenericEnum<A, B>()
	@Serializable
	@SerialName("VariantB")
	data class VariantB<A, B>(val content: B): GenericEnum<A, B>()
}

@Serializable
data class StructUsingGenericEnum (
	val enum_field: GenericEnum<String, Short>
)

@Serializable
sealed class GenericEnumUsingGenericEnum<T> {
	@Serializable
	@SerialName("VariantC")
	data class VariantC<T>(val content: GenericEnum<T, T>): GenericEnumUsingGenericEnum<T>()
	@Serializable
	@SerialName("VariantD")
	data class VariantD<T>(val content: GenericEnum<String, HashMap<String, T>>): GenericEnumUsingGenericEnum<T>()
	@Serializable
	@SerialName("VariantE")
	data class VariantE<T>(val content: GenericEnum<String, UInt>): GenericEnumUsingGenericEnum<T>()
}

/// Generated type representing the anonymous struct variant `VariantF` of the `GenericEnumsUsingStructVariants` Rust enum
@Serializable
data class GenericEnumsUsingStructVariantsVariantFInner<T> (
	val action: T
)

/// Generated type representing the anonymous struct variant `VariantG` of the `GenericEnumsUsingStructVariants` Rust enum
@Serializable
data class GenericEnumsUsingStructVariantsVariantGInner<T, U> (
	val action: T,
	val response: U
)

/// Generated type representing the anonymous struct variant `VariantH` of the `GenericEnumsUsingStructVariants` Rust enum
@Serializable
data class GenericEnumsUsingStructVariantsVariantHInner (
	val non_generic: Int
)

/// Generated type representing the anonymous struct variant `VariantI` of the `GenericEnumsUsingStructVariants` Rust enum
@Serializable
data class GenericEnumsUsingStructVariantsVariantIInner<T, U> (
	val vec: List<T>,
	val action: MyType<T, U>
)

@Serializable
sealed class GenericEnumsUsingStructVariants<T, U> {
	@Serializable
	@SerialName("VariantF")
	data class VariantF<T, U>(val content: GenericEnumsUsingStructVariantsVariantFInner<T>): GenericEnumsUsingStructVariants<T, U>()
	@Serializable
	@SerialName("VariantG")
	data class VariantG<T, U>(val content: GenericEnumsUsingStructVariantsVariantGInner<T, U>): GenericEnumsUsingStructVariants<T, U>()
	@Serializable
	@SerialName("VariantH")
	data class VariantH<T, U>(val content: GenericEnumsUsingStructVariantsVariantHInner): GenericEnumsUsingStructVariants<T, U>()
	@Serializable
	@SerialName("VariantI")
	data class VariantI<T, U>(val content: GenericEnumsUsingStructVariantsVariantIInner<T, U>): GenericEnumsUsingStructVariants<T, U>()
}

