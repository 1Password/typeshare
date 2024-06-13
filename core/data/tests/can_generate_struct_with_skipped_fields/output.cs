#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class MyStruct {
	[JsonProperty(Required = Required.Always)]
	public int A { get; set; }
	[JsonProperty(Required = Required.Always)]
	public int C { get; set; }
}

