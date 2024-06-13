#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(Red), "Red")]
[JsonSubtypes.KnownSubType(typeof(Banana), "Banana")]
[JsonSubtypes.KnownSubType(typeof(Vermont), "Vermont")]
public abstract record Options 
{
	public record Red(bool Content) : Options();
	public record Banana(string Content) : Options();
	public record Vermont(Options Content) : Options();
}


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
[JsonSubtypes.KnownSubType(typeof(News), "News")]
[JsonSubtypes.KnownSubType(typeof(Exactly), "Exactly")]
[JsonSubtypes.KnownSubType(typeof(Built), "Built")]
public abstract record MoreOptions 
{
	public record News(bool Content) : MoreOptions();
	public record exactly(MoreOptionsExactlyInner Content): MoreOptions();
	public record built(MoreOptionsBuiltInner Content): MoreOptions();
}


