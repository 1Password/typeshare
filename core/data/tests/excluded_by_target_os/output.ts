export interface MultipleTargets {
}

export enum SomeEnum {
}

export type TestEnum = 
	| { type: "Variant5", content?: undefined }
	| { type: "Variant7", content: {
}}
	| { type: "Variant8", content?: undefined };

