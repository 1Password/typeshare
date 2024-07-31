export interface DefinedTwice {
	field1: string;
}

export interface MultipleTargets {
}

export enum SomeEnum {
}

export type TestEnum = 
	| { type: "Variant1", content?: undefined }
	| { type: "Variant5", content?: undefined }
	| { type: "Variant7", content: {
	field1: string;
}}
	| { type: "Variant8", content?: undefined };

