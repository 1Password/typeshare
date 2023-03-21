@file:NoLiveLiterals

package com.agilebits.onepassword

import androidx.compose.runtime.NoLiveLiterals
import kotlinx.serialization.*

@Serializable
object AddressDetails

@Serializable
sealed class Address {
	@Serializable
	@SerialName("FixedAddress")
	data class FixedAddress(val content: AddressDetails): Address()
	@Serializable
	@SerialName("NoFixedAddress")
	object NoFixedAddress: Address()
}

