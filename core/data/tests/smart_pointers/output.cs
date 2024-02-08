#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

/** This is a comment. */
public class ArcyColors {
	public ushort Red { get; set; }
	public string Blue { get; set; }
	public IEnumerable<string> Green { get; set; }
}

/** This is a comment. */
public class MutexyColors {
	public IEnumerable<string> Blue { get; set; }
	public string Green { get; set; }
}

/** This is a comment. */
public class RcyColors {
	public string Red { get; set; }
	public IEnumerable<string> Blue { get; set; }
	public string Green { get; set; }
}

/** This is a comment. */
public class CellyColors {
	public string Red { get; set; }
	public IEnumerable<string> Blue { get; set; }
}

/** This is a comment. */
public class LockyColors {
	public string Red { get; set; }
}

/** This is a comment. */
public class CowyColors {
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


