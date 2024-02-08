#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class OtherType {
}

/** This is a comment. */
public class PersonTwo {
	public string name { get; set; }
	public ushort age { get; set; }
	public int extraSpecialFieldOne { get; set; }
	public IEnumerable<string>? extraSpecialFieldTwo { get; set; }
	public OtherType nonStandardDataType { get; set; }
	public IEnumerable<OtherType>? nonStandardDataTypeInArray { get; set; }
}

