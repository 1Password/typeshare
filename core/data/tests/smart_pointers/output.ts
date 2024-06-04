/** This is a comment. */
export interface ArcyColors {
	red: number;
	blue: string;
	green: string[];
}

/** This is a comment. */
export interface CellyColors {
	red: string;
	blue: string[];
}

/** This is a comment. */
export interface CowyColors {
	lifetime: string;
}

/** This is a comment. */
export interface LockyColors {
	red: string;
}

/** This is a comment. */
export interface MutexyColors {
	blue: string[];
	green: string;
}

/** This is a comment. */
export interface RcyColors {
	red: string;
	blue: string[];
	green: string;
}

/** This is a comment. */
export type BoxyColors = 
	| { type: "Red", content?: undefined }
	| { type: "Blue", content?: undefined }
	| { type: "Green", content: string };

