package com.agilebits

package object onepassword {

type UByte = Byte
type UShort = Short
type UInt = Int
type ULong = Int

}
package onepassword {

case class DisallowedType (
	disallowed_type: ULong,
	another_disallowed_type: Long,
	disallowed_type_serde_with: ULong
)

}
