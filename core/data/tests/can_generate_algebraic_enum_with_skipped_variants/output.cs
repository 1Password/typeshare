#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(A), "A")]
[JsonSubtypes.KnownSubType(typeof(C), "C")]
public abstract record SomeEnum 
{
	public record A(): SomeEnum();
	public record C(int Content) : SomeEnum();
}


