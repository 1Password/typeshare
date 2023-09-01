package com.agilebits

package object onepassword {

type UByte = Byte
type UShort = Short
type UInt = Int
type ULong = Int

}
package onepassword {

sealed trait SomeEnum {
	def serialName: String
}
object SomeEnum {
	case object A extends SomeEnum {
		val serialName: String = "A"
	}
	case class B(field1: String) extends SomeEnum {
		val serialName: String = "B"
	}
	case class C(field1: UInt, field2: Float) extends SomeEnum {
		val serialName: String = "C"
	}
	case class D(field3: Option[Boolean]) extends SomeEnum {
		val serialName: String = "D"
	}
}

}
