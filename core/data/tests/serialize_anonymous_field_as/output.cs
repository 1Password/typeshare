public record SomeEnum 
{
	/** The associated String contains some opaque context */
	public record Context(string Content) : SomeEnum();
	public record Other(int Content) : SomeEnum();
}


