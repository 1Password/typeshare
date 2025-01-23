#nullable enable

using System.Reflection;
using JsonSubTypes;
using Newtonsoft.Json;
using System.Runtime.Serialization;

public class EditItemViewModelSaveRequest {
	[JsonProperty(Required = Required.Always)]
	public string Context { get; set; }
	[JsonProperty(Required = Required.Always)]
	public IEnumerable<EditItemSaveValue> Values { get; set; }
	public AutoFillItemActionRequest? FillAction { get; set; }
}

