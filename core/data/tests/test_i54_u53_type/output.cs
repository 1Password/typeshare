#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class Foo {
	[JsonProperty(Required = Required.Always)]
	public long A { get; set; }
	[JsonProperty(Required = Required.Always)]
	public ulong B { get; set; }
}

