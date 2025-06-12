package com.agilebits

package object onepassword {

type UByte = Byte
type UShort = Short
type UInt = Int
type ULong = Int

}
package onepassword {

case class PointerSizedType (
	unsigned: ULong,
	signed: Long
)

}
