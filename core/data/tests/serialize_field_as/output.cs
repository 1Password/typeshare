#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class EditItemViewModelSaveRequest {
	public string Context { get; set; }
	public IEnumerable<EditItemSaveValue> Values { get; set; }
	public AutoFillItemActionRequest? FillAction { get; set; }
}

