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


