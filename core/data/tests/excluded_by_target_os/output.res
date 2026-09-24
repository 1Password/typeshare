/**
 * A struct with no target_os. Should be generated when
 * we use --target-os.
 */
type alwaysAccept = {
}

type definedTwice = {
	field1: string,
}

type excluded = {
}

type manyStruct = {
}

type multipleTargets = {
}

type nestedNotTarget1 = {
}

type otherExcluded = {
}

type alwaysAcceptEnum = 
	| @as("Variant1") Variant1
	| @as("Variant2") Variant2


type someEnum


type testEnumVariant7 = {
	field1: string,
}

type testEnumVariant9 = {
	field2: string,
}

@tag("type")
type testEnum = 
	| @as("Variant1") Variant1
	| @as("Variant5") Variant5
	| @as("Variant7") Variant7({ content: testEnumVariant7 })
	| @as("Variant8") Variant8
	| @as("Variant9") Variant9({ content: testEnumVariant9 })

