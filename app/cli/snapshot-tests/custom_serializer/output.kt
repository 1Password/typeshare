package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable(with = CustomSerializer::class)
data class BestHockeyTeams (
	val PittsburghPenguins: UInt,
	val Lies: String
)

/// Generated type representing the anonymous struct variant `NestedAnonymousStruct` of the `Phrase` Rust enum
@Serializable
data class PhraseNestedAnonymousStructInner (
	val nested_field: String,
	val another_nested_field: String
)

@Serializable(with = CustomSerializer::class)
sealed class Phrase {
	@Serializable
	@SerialName("ScanSetupCode")
	object ScanSetupCode: Phrase()
	@Serializable
	@SerialName("TotpSecondsRemaining")
	data class TotpSecondsRemaining(val content: String): Phrase()
	@Serializable
	@SerialName("NestedAnonymousStruct")
	data class NestedAnonymousStruct(val content: PhraseNestedAnonymousStructInner): Phrase()
}

