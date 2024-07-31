import Foundation

public struct DefinedTwice: Codable {
	public let field1: String

	public init(field1: String) {
		self.field1 = field1
	}
}

public struct MultipleTargets: Codable {
	public init() {}
}

public enum SomeEnum: String, Codable {
}


/// Generated type representing the anonymous struct variant `Variant7` of the `TestEnum` Rust enum
public struct TestEnumVariant7Inner: Codable {
	public let field1: String

	public init(field1: String) {
		self.field1 = field1
	}
}
public enum TestEnum: Codable {
	case variant1
	case variant5
	case variant7(TestEnumVariant7Inner)
	case variant8

	enum CodingKeys: String, CodingKey, Codable {
		case variant1 = "Variant1",
			variant5 = "Variant5",
			variant7 = "Variant7",
			variant8 = "Variant8"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .variant1:
				self = .variant1
				return
			case .variant5:
				self = .variant5
				return
			case .variant7:
				if let content = try? container.decode(TestEnumVariant7Inner.self, forKey: .content) {
					self = .variant7(content)
					return
				}
			case .variant8:
				self = .variant8
				return
			}
		}
		throw DecodingError.typeMismatch(TestEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for TestEnum"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .variant1:
			try container.encode(CodingKeys.variant1, forKey: .type)
		case .variant5:
			try container.encode(CodingKeys.variant5, forKey: .type)
		case .variant7(let content):
			try container.encode(CodingKeys.variant7, forKey: .type)
			try container.encode(content, forKey: .content)
		case .variant8:
			try container.encode(CodingKeys.variant8, forKey: .type)
		}
	}
}
