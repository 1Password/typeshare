package com.agilebits.onepassword

import kotlinx.serialization.Serializable
import kotlinx.serialization.SerialName

@Serializable
data class EditItemViewModelSaveRequest (
	val context: String,
	val values: List<EditItemSaveValue>,
	val fill_action: AutoFillItemActionRequest? = null
)

