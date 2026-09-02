/** Struct comment */
type itemDetailsFieldValue = {
}

/** Enum comment */
@tag("type")
type advancedColors = 
	/** This is a case comment */
	| @as("String") String({ content: string })
	| @as("Number") Number({ content: int })
	| @as("UnsignedNumber") UnsignedNumber({ content: int })
	| @as("NumberArray") NumberArray({ content: array<int> })
	/** Comment on the last element */
	| @as("ReallyCoolType") ReallyCoolType({ content: itemDetailsFieldValue })

@tag("type")
type advancedColors2 = 
	/** This is a case comment */
	| @as("string") String({ content: string })
	| @as("number") Number({ content: int })
	| @as("number-array") NumberArray({ content: array<int> })
	/** Comment on the last element */
	| @as("really-cool-type") ReallyCoolType({ content: itemDetailsFieldValue })

