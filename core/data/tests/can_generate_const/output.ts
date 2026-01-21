export const MY_INT_VAR: number = 12;
export const EMPTY: string = ``;
export const SIMPLE_ASCII: string = `Hello, world!`;
export const MULTILINE: string = `Line1
Line2
Line3`;
export const ESCAPED_CHARACTERS: string = `First\\line.
Second "quoted" line.	End.`;
export const UNICODE: string = `Emoji: 😄, Accented: café, Chinese: 世界`;
export const RAW_STRING: string = String.raw`Raw \n, "quotes" are okay, and single \ is fine too`;
export const CONTAINS_BACKTICK: string = `Backtick: \` inside`;
export const CONTAINS_DOLLAR_CURLY: string = `\${not_interpolation}`;
export const ENDS_WITH_ODD_BACKSLASH: string = String.raw`Odd number of backslashes: \\` + '\\';
export const NULL_BYTE: string = `Null:\u0000End`;
export const COMBINING: string = `é vs é`;
