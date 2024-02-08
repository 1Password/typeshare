#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class CustomType {
}

public class Types {
	public string S { get; set; }
	public string StaticS { get; set; }
	public short Int8 { get; set; }
	public float Float { get; set; }
	public double Double { get; set; }
	public IEnumerable<string> Array { get; set; }
	public string[] FixedLengthArray { get; set; }
	public IDictionary<string, int> Dictionary { get; set; }
	public IDictionary<string, int>? OptionalDictionary { get; set; }
	public CustomType CustomType { get; set; }
}

