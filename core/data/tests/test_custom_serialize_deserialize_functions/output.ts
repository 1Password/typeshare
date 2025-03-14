export interface Foo {
	time: Date;
	time2: Date;
	time3: Date;
	bytes: Uint8Array;
	bytes2: Uint8Array;
}

export interface TwoFoo {
	time: Date;
	bytes: Uint8Array;
}

/**
 * Custom JSON reviver and replacer functions for dynamic data transformation
 * ReviverFunc is used during JSON parsing to detect and transform specific data structures
 * ReplacerFunc is used during JSON serialization to modify certain values before stringifying.
 * These functions allow for flexible encoding and decoding of data, ensuring that complex types are properly handled when converting between TS objects and JSON
 */
export const ReviverFunc = (key: string, value: unknown): unknown => {
    if (typeof value === "string" && /^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?Z$/.test(value) && (key == "time" || key == "time2" || key == "time3")) {
        return new Date(value);
    }
    if (Array.isArray(value) && value.every(v => Number.isInteger(v) && v >= 0 && v <= 255) && value.length > 0)  {
        return new Uint8Array(value);
    }
    return value;
};

export const ReplacerFunc = (key: string, value: unknown): unknown => {
    if (value instanceof Date) {
        return value.toISOString();
    }
    if (value instanceof Uint8Array) {
        return Array.from(value);
    }
    return value;
};
