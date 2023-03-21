import Foundation

/// This is a comment.
public enum Colors: Codable {
	case red
	case blue
	case green(String)

	enum CodingKeys: String, CodingKey, Codable {
		case red = "Red",
			blue = "Blue",
			green = "Green"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .red:
				self = .red
				return
			case .blue:
				self = .blue
				return
			case .green:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .green(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(Colors.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for Colors"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .red:
			try container.encode(CodingKeys.red, forKey: .type)
		case .blue:
			try container.encode(CodingKeys.blue, forKey: .type)
		case .green(let content):
			try container.encode(CodingKeys.green, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
