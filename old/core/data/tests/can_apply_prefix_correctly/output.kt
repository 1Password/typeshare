@Serializable
data class OPItemDetailsFieldValue (
	val hello: String
)

@Serializable
sealed class OPAdvancedColors {
	@Serializable
	@SerialName("String")
	data class String(val c: String): OPAdvancedColors()
	@Serializable
	@SerialName("Number")
	data class Number(val c: Int): OPAdvancedColors()
	@Serializable
	@SerialName("NumberArray")
	data class NumberArray(val c: List<Int>): OPAdvancedColors()
	@Serializable
	@SerialName("ReallyCoolType")
	data class ReallyCoolType(val c: OPItemDetailsFieldValue): OPAdvancedColors()
	@Serializable
	@SerialName("ArrayReallyCoolType")
	data class ArrayReallyCoolType(val c: List<OPItemDetailsFieldValue>): OPAdvancedColors()
	@Serializable
	@SerialName("DictionaryReallyCoolType")
	data class DictionaryReallyCoolType(val c: HashMap<String, OPItemDetailsFieldValue>): OPAdvancedColors()
}

