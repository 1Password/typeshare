#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class A {
	[JsonProperty(Required = Required.Always)]
	public uint Field { get; set; }
}

public class ABC {
	[JsonProperty(Required = Required.Always)]
	public uint Field { get; set; }
}

public class AB {
	[JsonProperty(Required = Required.Always)]
	public uint Field { get; set; }
}

public class OutsideOfModules {
	[JsonProperty(Required = Required.Always)]
	public uint Field { get; set; }
}

