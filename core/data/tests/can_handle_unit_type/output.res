/** This struct has a unit field */
type structHasVoidType = {
	thisIsAUnit: unit,
}

/** This enum has a variant associated with unit data */
@tag("type")
type enumHasVoidType = 
	| @as("hasAUnit") HasAUnit({ content: unit })

