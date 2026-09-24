/** This is a comment. */
type arcyColors = {
	red: int,
	blue: string,
	green: array<string>,
}

/** This is a comment. */
type cellyColors = {
	red: string,
	blue: array<string>,
}

/** This is a comment. */
type cowyColors = {
	lifetime: string,
}

/** This is a comment. */
type lockyColors = {
	red: string,
}

/** This is a comment. */
type mutexyColors = {
	blue: array<string>,
	green: string,
}

/** This is a comment. */
type rcyColors = {
	red: string,
	blue: array<string>,
	green: string,
}

/** This is a comment. */
@tag("type")
type boxyColors = 
	| @as("Red") Red
	| @as("Blue") Blue
	| @as("Green") Green({ content: string })

