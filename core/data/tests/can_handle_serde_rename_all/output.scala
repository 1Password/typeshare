/**
 * Generated by typeshare 1.1.0
 */
package com.agilebits

package object onepassword {

type UByte = Byte
type UShort = Short
type UInt = Int
type ULong = Int

}
package onepassword {

// This is a Person struct with camelCase rename
case class Person (
	firstName: String,
	lastName: String,
	age: UByte,
	extraSpecialField1: Int,
	extraSpecialField2: Option[Vector[String]] = None
)

// This is a Person2 struct with UPPERCASE rename
case class Person2 (
	FIRST_NAME: String,
	LAST_NAME: String,
	AGE: UByte
)

}
