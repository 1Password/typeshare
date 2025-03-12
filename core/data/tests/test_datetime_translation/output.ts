export interface Foo {
	time: Date;
	time2: Date;
	time3: Date;
	nonTime: string;
}

/**
 * Custom JSON reviver and replacer functions for dynamic data transformation
 * ReviverFunc is used during JSON parsing to detect and transform specific data structures
 * ReplacerFunc is used during JSON serialization to modify certain values before stringifying.
 * These functions allow for flexible encoding and decoding of data, ensuring that complex types are properly handled when converting between TS objects and JSON
 */
export const ReviverFunc = (key: string, value: unknown): unknown => {
    if (/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/.test(value as string) && (key == "time" || key == "time2" || key == "time3")) {
        return new Date(value as string);
    }
    return value;
};

export const ReplacerFunc = (key: string, value: unknown): unknown => {
    if (value instanceof Date) {
        return value.toISOString();
    }
    return value;
};
