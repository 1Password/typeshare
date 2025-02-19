export interface Foo {
	thisIsBits: Uint8Array;
}

export function TypeshareReviver(key: string, value: unknown): unknown {
    return Array.isArray(value) && value.every(v => Number.isFinite(v) && v >= 0 && v <= 255)  
        ? new Uint8Array(value) 
        : value;
}

export function TypeshareReplacer(key: string, value: unknown): unknown {
    if (value instanceof Uint8Array) {
        return Array.from(value);
    }
    return value;
}
