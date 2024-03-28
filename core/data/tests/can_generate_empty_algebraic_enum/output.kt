@Serializable
object OPAddressDetails

@Serializable
sealed class OPAddress {
	@Serializable
	@SerialName("FixedAddress")
	data class FixedAddress(val content: AddressDetails): Address()
	@Serializable
	@SerialName("NoFixedAddress")
	object NoFixedAddress: Address()
}

