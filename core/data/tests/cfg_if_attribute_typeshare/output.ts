export type Bytes = number[];

/**
 * Example of a type that is conditionally typeshared
 * based on a feature "typeshare-support". This does not
 * conditionally typeshare but allows a conditionally
 * typeshared type to generate typeshare types when behind
 * a `cfg_attr` condition.
 */
export interface TestStruct1 {
	field: string;
}

export interface TestStruct2<R> {
	field1: string;
	field2: R;
}

export interface TestStruct3 {
	field_1: string;
}

