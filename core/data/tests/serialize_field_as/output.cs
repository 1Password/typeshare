public class EditItemViewModelSaveRequest {
	public string Context { get; set; }
	public IEnumerable<EditItemSaveValue> Values { get; set; }
	public AutoFillItemActionRequest? FillAction { get; set; }
}

