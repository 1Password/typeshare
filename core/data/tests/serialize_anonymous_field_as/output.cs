#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(Context), "Context")]
[JsonSubtypes.KnownSubType(typeof(Other), "Other")]
public abstract record SomeEnum 
{
	/** The associated String contains some opaque context */
	public record Context(string Content) : SomeEnum();
	public record Other(int Content) : SomeEnum();
}


