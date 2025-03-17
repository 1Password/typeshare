package com.agilebits.onepassword

public record EditItemViewModelSaveRequest(
	String context,
	java.util.ArrayList<EditItemSaveValue> values,
	AutoFillItemActionRequest fill_action
) {}

