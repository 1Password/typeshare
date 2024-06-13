#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/** Generated type representing the anonymous struct variant `List` of the `AnonymousStructWithRename` Rust enum */
public class AnonymousStructWithRenameListInner {
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> list { get; set; }
}

/** Generated type representing the anonymous struct variant `LongFieldNames` of the `AnonymousStructWithRename` Rust enum */
public class AnonymousStructWithRenameLongFieldNamesInner {
	[JsonProperty(Required = Required.Always)]
	public string some_long_field_name { get; set; }
	[JsonProperty(Required = Required.Always)]
	public bool and { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> but_one_more { get; set; }
}

/** Generated type representing the anonymous struct variant `KebabCase` of the `AnonymousStructWithRename` Rust enum */
public class AnonymousStructWithRenameKebabCaseInner {
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> another-list { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string camelCaseStringField { get; set; }
	[JsonProperty(Required = Required.Always)]
	public bool something-else { get; set; }
}

[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(List), "List")]
[JsonSubtypes.KnownSubType(typeof(LongFieldNames), "LongFieldNames")]
[JsonSubtypes.KnownSubType(typeof(KebabCase), "KebabCase")]
public abstract record AnonymousStructWithRename 
{
	public record list(AnonymousStructWithRenameListInner Content): AnonymousStructWithRename();
	public record longFieldNames(AnonymousStructWithRenameLongFieldNamesInner Content): AnonymousStructWithRename();
	public record kebabCase(AnonymousStructWithRenameKebabCaseInner Content): AnonymousStructWithRename();
}


