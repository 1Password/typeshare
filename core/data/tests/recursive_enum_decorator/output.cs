public record Options 
{
	public record Red(bool Content) : Options();
	public record Banana(string Content) : Options();
	public record Vermont(Options Content) : Options();
}


/** Generated type representing the anonymous struct variant `Exactly` of the `MoreOptions` Rust enum */
public class MoreOptionsExactlyInner {
	public string Config { get; set; }
}

/** Generated type representing the anonymous struct variant `Built` of the `MoreOptions` Rust enum */
public class MoreOptionsBuiltInner {
	public MoreOptions Top { get; set; }
}

public record MoreOptions 
{
	public record News(bool Content) : MoreOptions();
	public record exactly(MoreOptionsExactlyInner Content): MoreOptions();
	public record built(MoreOptionsBuiltInner Content): MoreOptions();
}


