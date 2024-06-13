#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class Foo {
	[JsonProperty(Required = Required.Always)]
	public string Time { get; set; }
}

