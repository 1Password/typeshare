#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class SomeStruct {
	[JsonProperty(Required = Required.Always)]
	public uint FieldA { get; set; }
}

