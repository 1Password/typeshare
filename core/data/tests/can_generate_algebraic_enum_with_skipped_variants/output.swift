import Foundation

public enum SomeEnum: Codable {
	case a
	case c(Int32)

	enum CodingKeys: String, CodingKey, Codable {
		case a = "A",
			c = "C"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .a:
				self = .a
				return
			case .c:
				if let content = try? container.decode(Int32.self, forKey: .content) {
					self = .c(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(SomeEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for SomeEnum"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .a:
			try container.encode(CodingKeys.a, forKey: .type)
		case .c(let content):
			try container.encode(CodingKeys.c, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
