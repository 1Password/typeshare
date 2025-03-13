/**
 * A struct with no target_os. Should be generated when
 * we use --target-os.
 */
export interface AlwaysAccept {
}

export interface DefinedTwice {
	field1: string;
}

export interface Excluded {
}

export interface ManyStruct {
}

export interface MultipleTargets {
}

export interface NestedNotTarget1 {
}

export interface OtherExcluded {
}

export enum AlwaysAcceptEnum {
	Variant1 = "Variant1",
	Variant2 = "Variant2",
}

export enum SomeEnum {
}

export type TestEnum = 
	| { type: "Variant1", content?: undefined }
	| { type: "Variant5", content?: undefined }
	| { type: "Variant7", content: {
	field1: string;
}}
	| { type: "Variant8", content?: undefined }
	| { type: "Variant9", content: {
	field2: string;
}};

