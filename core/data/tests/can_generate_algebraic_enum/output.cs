#nullable enable

using System;
using System.Reflection;
using System.Collections.Generic;

namespace Company.Domain.Models;

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

/** Struct comment */
public class ItemDetailsFieldValue {
}

/** Enum comment */
public record AdvancedColors 
{
	/** This is a case comment */
	public record String(string Content) : AdvancedColors();
	public record Number(int Content) : AdvancedColors();
	public record UnsignedNumber(uint Content) : AdvancedColors();
	public record NumberArray(IEnumerable<int> Content) : AdvancedColors();
	/** Comment on the last element */
	public record ReallyCoolType(ItemDetailsFieldValue Content) : AdvancedColors();
}


public record AdvancedColors2 
{
	/** This is a case comment */
	public record String(string Content) : AdvancedColors2();
	public record Number(int Content) : AdvancedColors2();
	public record NumberArray(IEnumerable<int> Content) : AdvancedColors2();
	/** Comment on the last element */
	public record ReallyCoolType(ItemDetailsFieldValue Content) : AdvancedColors2();
}


