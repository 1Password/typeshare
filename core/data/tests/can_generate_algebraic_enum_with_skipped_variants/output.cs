public record SomeEnum 
{
	public record A(): SomeEnum();
	public record C(int Content) : SomeEnum();
}


