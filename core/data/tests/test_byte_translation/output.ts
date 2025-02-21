export interface Foo {
	thisIsBits: Uint8Array;
}

export function ReviverFunc(key: string, value: unknown): unknown {
    if (Array.isArray(value) && value.every(v => Number.isInteger(v) && v >= 0 && v <= 255)) {
                    return new Uint8Array(value);
                }
    return value;
}

export function ReplacerFunc(key: string, value: unknown): unknown {
    if (value instanceof Uint8Array) {
        return Array.from(value);
    }
    return value;
}
