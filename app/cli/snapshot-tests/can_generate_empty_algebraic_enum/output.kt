package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
object OPAddressDetails

@Serializable
sealed class OPAddress {
	@Serializable
	@SerialName("FixedAddress")
	data class FixedAddress(val content: OPAddressDetails): OPAddress()
	@Serializable
	@SerialName("NoFixedAddress")
	object NoFixedAddress: OPAddress()
}

