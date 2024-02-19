#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/** This is a comment. */
public enum Colors
{
	[EnumMember(Value = "red")]
	Red,

	[EnumMember(Value = "blue-ish")]
	Blue,

	Green,

}

