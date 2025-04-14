package com.agilebits

package onepassword {

// This struct has a unit field
case class StructHasVoidType (
	thisIsAUnit: Unit
)

// This enum has a variant associated with unit data
sealed trait EnumHasVoidType {
	def serialName: String
}
object EnumHasVoidType {
	case class HasAUnit(content: Unit) extends EnumHasVoidType {
		val serialName: String = "hasAUnit"
	}
}

}
