#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class MyType {
	[JsonProperty(Required = Required.Always)]
	public char Field { get; set; }
}

