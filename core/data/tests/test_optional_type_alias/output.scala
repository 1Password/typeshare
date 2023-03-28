package com.agilebits

package object onepassword {

type UByte = Byte
type UShort = Short
type UInt = Int
type ULong = Int

type OptionalU32 = Option[UInt]

type OptionalU16 = Option[UShort]

}
package onepassword {

case class FooBar (
	foo: OptionalU32,
	bar: OptionalU16
)

}
