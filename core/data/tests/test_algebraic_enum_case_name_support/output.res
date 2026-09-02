type itemDetailsFieldValue = {
}

@tag("type")
type advancedColors = 
	| @as("string") String({ content: string })
	| @as("number") Number({ content: int })
	| @as("number-array") NumberArray({ content: array<int> })
	| @as("reallyCoolType") ReallyCoolType({ content: itemDetailsFieldValue })

