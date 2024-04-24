package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

typealias GenericTypeAlias<T> = List<T>

typealias NonGenericAlias = GenericTypeAlias<String?>

