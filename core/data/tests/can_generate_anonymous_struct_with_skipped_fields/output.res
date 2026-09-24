/** Enum keeping track of who autofilled a field */
/** This field was autofilled by us */
type autofilledByUs = {
	/** The UUID for the fill */
	uuid: string,
}

/** Something else autofilled this field */
type autofilledBySomethingElse = {
	/** The UUID for the fill */
	uuid: string,
}

@tag("type")
type autofilledBy = 
	/** This field was autofilled by us */
	| @as("Us") Us({ content: autofilledByUs })
	/** Something else autofilled this field */
	| @as("SomethingElse") SomethingElse({ content: autofilledBySomethingElse })

