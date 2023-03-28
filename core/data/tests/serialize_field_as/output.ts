export interface EditItemViewModelSaveRequest {
	context: string;
	values: EditItemSaveValue[];
	fill_action?: AutoFillItemActionRequest;
}

