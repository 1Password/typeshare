#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/**
 * This is a comment.
 * Continued lovingly here
 */
public enum Colors
{
	[EnumMember(Value = "red")]
	Red,

	[EnumMember(Value = "blue")]
	Blue,

	/** Green is a cool color */
	[EnumMember(Value = "green-like")]
	Green,

}

