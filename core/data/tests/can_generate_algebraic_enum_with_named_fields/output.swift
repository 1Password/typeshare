import Foundation

public enum OPSomeEnum: Codable {
	case a
	case b(field1: String)
	case c(field1: UInt32, field2: Float)
	case d(field3: Bool?)

	enum CodingKeys: String, CodingKey, Codable {
		case a = "A",
			b = "B",
			c = "C",
			d = "D"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, field1, field2, field3
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .a:
				self = .a
				return
			case .b:
				if
					let field1 = try? container.decode(String.self, forKey: .field1)
				{
					self = .b(field1: field1)
					return
				}
			case .c:
				if
					let field1 = try? container.decode(UInt32.self, forKey: .field1),
					let field2 = try? container.decode(Float.self, forKey: .field2)
				{
					self = .c(field1: field1, field2: field2)
					return
				}
			case .d:
				let field3 = try? container.decodeIfPresent(Bool?.self, forKey: .field3)
				self = .d(field3: field3)
				return
			}
		}
		throw DecodingError.typeMismatch(OPSomeEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPSomeEnum"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .a:
			try container.encode(CodingKeys.a, forKey: .type)
		case .b(let field1):
			try container.encode(CodingKeys.b, forKey: .type)
			try container.encode(field1, forKey: .field1)
		case .c(let field1, let field2):
			try container.encode(CodingKeys.c, forKey: .type)
			try container.encode(field1, forKey: .field1)
			try container.encode(field2, forKey: .field2)
		case .d(let field3):
			try container.encode(CodingKeys.d, forKey: .type)
			try container.encode(field3, forKey: .field3)
		}
	}
}
