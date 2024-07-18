package com.agilebits

package object onepassword {

type UByte = Byte
type UShort = Short
type UInt = Int
type ULong = Int

}
package onepassword {

case class A (
	field: UInt
)

case class AB (
	field: UInt
)

case class ABC (
	field: UInt
)

case class OutsideOfModules (
	field: UInt
)

}
