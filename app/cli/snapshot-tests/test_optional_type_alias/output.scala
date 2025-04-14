package com.agilebits

package object onepassword {

type UByte = Byte
type UShort = Short
type UInt = Int
type ULong = Int

type OptionalU16 = Option[UShort]

type OptionalU32 = Option[UInt]

}
package onepassword {

case class FooBar (
	foo: OptionalU32,
	bar: OptionalU16
)

}
