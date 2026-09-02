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
	/** Some other thing */
	thing: int,
}

@tag("type")
type autofilledBy = 
	/** This field was autofilled by us */
	| @as("Us") Us({ content: autofilledByUs })
	/** Something else autofilled this field */
	| @as("SomethingElse") SomethingElse({ content: autofilledBySomethingElse })

/** This is a comment (yareek sameek wuz here) */
type enumWithManyVariantsAnonVariant = {
	uuid: string,
}

type enumWithManyVariantsAnotherAnonVariant = {
	uuid: string,
	thing: int,
}

@tag("type")
type enumWithManyVariants = 
	| @as("UnitVariant") UnitVariant
	| @as("TupleVariantString") TupleVariantString({ content: string })
	| @as("AnonVariant") AnonVariant({ content: enumWithManyVariantsAnonVariant })
	| @as("TupleVariantInt") TupleVariantInt({ content: int })
	| @as("AnotherUnitVariant") AnotherUnitVariant
	| @as("AnotherAnonVariant") AnotherAnonVariant({ content: enumWithManyVariantsAnotherAnonVariant })

