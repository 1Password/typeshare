#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/** This is a comment. */
public class ArcyColors {
	[JsonProperty(Required = Required.Always)]
	public ushort Red { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string Blue { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> Green { get; set; }
}

/** This is a comment. */
public class MutexyColors {
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> Blue { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string Green { get; set; }
}

/** This is a comment. */
public class RcyColors {
	[JsonProperty(Required = Required.Always)]
	public string Red { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> Blue { get; set; }
	[JsonProperty(Required = Required.Always)]
	public string Green { get; set; }
}

/** This is a comment. */
public class CellyColors {
	[JsonProperty(Required = Required.Always)]
	public string Red { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<string> Blue { get; set; }
}

/** This is a comment. */
public class LockyColors {
	[JsonProperty(Required = Required.Always)]
	public string Red { get; set; }
}

/** This is a comment. */
public class CowyColors {
	[JsonProperty(Required = Required.Always)]
	public string Lifetime { get; set; }
}

/** This is a comment. */
[JsonConverter(typeof(JsonSubtypes), "type")]
[JsonSubtypes.KnownSubType(typeof(Red), "Red")]
[JsonSubtypes.KnownSubType(typeof(Blue), "Blue")]
[JsonSubtypes.KnownSubType(typeof(Green), "Green")]
public abstract record BoxyColors 
{
	public record Red(): BoxyColors();
	public record Blue(): BoxyColors();
	public record Green(string Content) : BoxyColors();
}


