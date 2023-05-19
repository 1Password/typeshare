package com.agilebits

package onepassword {

case class OverrideStruct (
	fieldToOverride: Short
)

// Generated type representing the anonymous struct variant `AnonymousStructVariant` of the `OverrideEnum` Rust enum
case class OverrideEnumAnonymousStructVariantInner (
	fieldToOverride: Short
)

sealed trait OverrideEnum {
	def serialName: String
}
object OverrideEnum {
	case object UnitVariant extends OverrideEnum {
		val serialName: String = "UnitVariant"
	}
	case class TupleVariant(content: String) extends OverrideEnum {
		val serialName: String = "TupleVariant"
	}
	case class AnonymousStructVariant(content: OverrideEnumAnonymousStructVariantInner) extends OverrideEnum {
		val serialName: String = "AnonymousStructVariant"
	}
}

}
