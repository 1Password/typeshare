public class QualifiedTypes {
	public string Unqualified { get; set; }
	public string Qualified { get; set; }
	public IEnumerable<string> QualifiedVec { get; set; }
	public IDictionary<string, string> QualifiedHashmap { get; set; }
	public string? QualifiedOptional { get; set; }
	public IDictionary<string, IEnumerable<string>>? QualfiedOptionalHashmapVec { get; set; }
}

