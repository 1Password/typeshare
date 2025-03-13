import Foundation

public typealias AliasTest = [SomethingFoo]

public struct Test: Codable {
	public let field1: SomethingFoo
	public let field2: SomethingFoo?

	public init(field1: SomethingFoo, field2: SomethingFoo?) {
		self.field1 = field1
		self.field2 = field2
	}
}

public enum SomethingFoo: String, Codable {
	case a = "A"
}

public enum Parent: Codable {
	case b(SomethingFoo)

	enum CodingKeys: String, CodingKey, Codable {
		case b = "B"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, value
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .b:
				if let content = try? container.decode(SomethingFoo.self, forKey: .value) {
					self = .b(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(Parent.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for Parent"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .b(let content):
			try container.encode(CodingKeys.b, forKey: .type)
			try container.encode(content, forKey: .value)
		}
	}
}
