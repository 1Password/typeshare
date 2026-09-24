@tag("type")
type genericEnum<'a, 'b> = 
	| @as("VariantA") VariantA({ content: 'a })
	| @as("VariantB") VariantB({ content: 'b })

type structUsingGenericEnum = {
	enum_field: genericEnum<string, int>,
}

@tag("type")
type genericEnumUsingGenericEnum<'t> = 
	| @as("VariantC") VariantC({ content: genericEnum<'t, 't> })
	| @as("VariantD") VariantD({ content: genericEnum<string, Dict.t<'t>> })
	| @as("VariantE") VariantE({ content: genericEnum<string, int> })

type genericEnumsUsingStructVariantsVariantF<'t, 'u> = {
	action: 't,
}

type genericEnumsUsingStructVariantsVariantG<'t, 'u> = {
	action: 't,
	response: 'u,
}

type genericEnumsUsingStructVariantsVariantH<'t, 'u> = {
	non_generic: int,
}

type genericEnumsUsingStructVariantsVariantI<'t, 'u> = {
	vec: array<'t>,
	action: myType<'t, 'u>,
}

@tag("type")
type genericEnumsUsingStructVariants<'t, 'u> = 
	| @as("VariantF") VariantF({ content: genericEnumsUsingStructVariantsVariantF<'t, 'u> })
	| @as("VariantG") VariantG({ content: genericEnumsUsingStructVariantsVariantG<'t, 'u> })
	| @as("VariantH") VariantH({ content: genericEnumsUsingStructVariantsVariantH<'t, 'u> })
	| @as("VariantI") VariantI({ content: genericEnumsUsingStructVariantsVariantI<'t, 'u> })

