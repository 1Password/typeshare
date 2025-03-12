export interface Foo {
	time: Date;
}

export function ReviverFunc(key: string, value: unknown): unknown {
    if (/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$/.test(value as string)) {
        return new Date(value as string)
    }
    return value;
}

export function ReplacerFunc(key: string, value: unknown): unknown {
    if (value instanceof Date) {
        return value.toISOString();
    }
    return value;
}
