export interface Foo {
	bytes: Uint8Array;
}

// Reviver code  - required for JSON deserialization
export function TypeshareReviver(key: string, value: unknown): unknown {
    return Array.isArray(value) && value.every(Number.isFinite) 
        ? new Uint8Array(value) 
        : value;
}
    
// Replacer code - required for JSON serialization
export function TypeshareReplacer(key: string, value: unknown): unknown {
    if (value instanceof Uint8Array) {
        return Array.from(value);
    }
    return value;
}
