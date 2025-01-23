#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/** Generated type representing the anonymous struct variant `Exactly` of the `MoreOptions` Rust enum */
public class MoreOptionsExactlyInner {
	[JsonProperty(Required = Required.Always)]
	public string Config { get; set; }
}

/** Generated type representing the anonymous struct variant `Built` of the `MoreOptions` Rust enum */
public class MoreOptionsBuiltInner {
	[JsonProperty(Required = Required.Always)]
	public MoreOptions Top { get; set; }
}

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(News), "news")]
[JsonSubtypes.KnownSubType(typeof(Exactly), "exactly")]
[JsonSubtypes.KnownSubType(typeof(Built), "built")]
public abstract record MoreOptions 
{
	public record News(bool Content) : MoreOptions();
	public record Exactly(MoreOptionsExactlyInner Content): MoreOptions();
	public record Built(MoreOptionsBuiltInner Content): MoreOptions();
}


[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(Red), "red")]
[JsonSubtypes.KnownSubType(typeof(Banana), "banana")]
[JsonSubtypes.KnownSubType(typeof(Vermont), "vermont")]
public abstract record Options 
{
	public record Red(bool Content) : Options();
	public record Banana(string Content) : Options();
	public record Vermont(Options Content) : Options();
}


