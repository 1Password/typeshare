type genericStruct<'a, 'b> = {
	field_a: 'a,
	field_b: array<'b>,
}

type genericStructUsingGenericStruct<'t> = {
	struct_field: genericStruct<string, 't>,
	second_struct_field: genericStruct<'t, string>,
	third_struct_field: genericStruct<'t, array<'t>>,
}

@tag("type")
type enumUsingGenericStruct = 
	| @as("VariantA") VariantA({ content: genericStruct<string, float> })
	| @as("VariantB") VariantB({ content: genericStruct<string, int> })
	| @as("VariantC") VariantC({ content: genericStruct<string, bool> })
	| @as("VariantD") VariantD({ content: genericStructUsingGenericStruct<unit> })

