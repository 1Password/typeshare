#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class Foo {
	public long A { get; set; }
	public ulong B { get; set; }
}

