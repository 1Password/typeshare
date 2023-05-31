@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

@Serializable
data class Foo (
	val time: java.time.Instant
)


object JavaInstantSerializer : KSerializer<java.time.Instant> {
    override val descriptor = PrimitiveDescriptor("Instant", PrimitiveKind.STRING)
    override fun serialize(encoder: Encoder, value: java.time.Instant) = encoder.encodeString(value)
    override fun deserialize(decoder: Decoder): java.time.Instant = java.time.Instant.parse(decoder.decodeString())
}

