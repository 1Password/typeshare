type overrideStruct = {
	fieldToOverride: int,
}

type overrideEnumAnonymousStructVariant = {
	fieldToOverride: int,
}

@tag("type")
type overrideEnum = 
	| @as("UnitVariant") UnitVariant
	| @as("TupleVariant") TupleVariant({ content: string })
	| @as("AnonymousStructVariant") AnonymousStructVariant({ content: overrideEnumAnonymousStructVariant })

