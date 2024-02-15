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

public class ItemDetailsFieldValue {
	public string Hello { get; set; }
}

public record AdvancedColors 
{
	public record String(string C) : AdvancedColors();
	public record Number(int C) : AdvancedColors();
	public record NumberArray(IEnumerable<int> C) : AdvancedColors();
	public record ReallyCoolType(ItemDetailsFieldValue C) : AdvancedColors();
	public record ArrayReallyCoolType(IEnumerable<ItemDetailsFieldValue> C) : AdvancedColors();
	public record DictionaryReallyCoolType(IDictionary<string, ItemDetailsFieldValue> C) : AdvancedColors();
}


