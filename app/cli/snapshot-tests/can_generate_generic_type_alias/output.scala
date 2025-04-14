package com.agilebits

package object onepassword {

type GenericTypeAlias[T] = Vector[T]

type NonGenericAlias = GenericTypeAlias[Option[String]]

}
