package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

typealias Bytes = List<UByte>

@Serializable
@JvmInline
value class TestStruct3(
	private val value: String
) {
	fun unwrap() = value

	override fun toString(): String = "***"
}

/// Example of a type that is conditionally typeshared
/// based on a feature "typeshare-support". This does not
/// conditionally typeshare but allows a conditionally
/// typeshared type to generate typeshare types when behind
/// a `cfg_attr` condition.
@Serializable
data class TestStruct1 (
	val field: String
)

@Serializable
data class TestStruct2<R> (
	val field1: String,
	val field2: R
)

@Serializable
data class TestStruct4 (
	val field: Long
)

