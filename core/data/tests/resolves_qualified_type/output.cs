#nullable enable

using System;
using System.Reflection;
using System.Collections.Generic;

class EnumLabelAttribute : Attribute
{
    public string Label { get; }

    public EnumLabelAttribute(string label)
    {
        Label = label;
    }
}

public static class EnumExtensions
{
    public static string Label<T>(this T value)
        where T : Enum
    {
        var fieldName = value.ToString();
        var field = typeof(T).GetField(fieldName, BindingFlags.Public | BindingFlags.Static);
        return field?.GetCustomAttribute<EnumLabelAttribute>()?.Label ?? fieldName;
    }
}

public class QualifiedTypes {
	public string Unqualified { get; set; }
	public string Qualified { get; set; }
	public IEnumerable<string> QualifiedVec { get; set; }
	public IDictionary<string, string> QualifiedHashmap { get; set; }
	public string? QualifiedOptional { get; set; }
	public IDictionary<string, IEnumerable<string>>? QualfiedOptionalHashmapVec { get; set; }
}

