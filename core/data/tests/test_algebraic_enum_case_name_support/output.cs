public class ItemDetailsFieldValue {
}

public record AdvancedColors 
{
	public record String(string Content) : AdvancedColors();
	public record Number(int Content) : AdvancedColors();
	public record NumberArray(IEnumerable<int> Content) : AdvancedColors();
	public record ReallyCoolType(ItemDetailsFieldValue Content) : AdvancedColors();
}


