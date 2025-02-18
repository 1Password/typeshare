export interface Foo {
	bytes: Uint8Array;
}

// Reviver code  - required for JSON deserialization
function TypeshareReviver(key: string, value: unknown): unknown { 
    return isNumberArray(value) ? new Uint8Array(value) : value; 
}
            
function isNumberArray(value: unknown): value is number[] {
    return Array.isArray(value) && value.every(item => typeof item === "number");
}
            
// Replacer code - required for JSON serialization
            
function TypeshareReplacer(key: string, value: unknown): unknown {
    if (value instanceof Uint8Array) {
        return Array.from(value);
    }
    return value;
}
