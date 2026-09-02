type addressDetails = {
}

@tag("type")
type address = 
	| @as("FixedAddress") FixedAddress({ content: addressDetails })
	| @as("NoFixedAddress") NoFixedAddress

