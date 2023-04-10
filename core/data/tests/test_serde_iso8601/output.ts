export interface Foo {
	time: Date;
}

export function TypeshareDateReviver(key, value): Date { return new Date(value); }
