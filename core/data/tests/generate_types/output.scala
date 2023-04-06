package com.agilebits

package onepassword {

class CustomType extends Serializable

case class Types (
	s: String,
	static_s: String,
	int8: Byte,
	float: Float,
	double: Double,
	array: Vector[String],
	fixed_length_array: Vector[String],
	dictionary: Map[String, Int],
	optional_dictionary: Option[Map[String, Int]] = None,
	custom_type: CustomType
)

}
