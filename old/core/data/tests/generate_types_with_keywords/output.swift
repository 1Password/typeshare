import Foundation

public struct `catch`: Codable {
	public let `default`: String
	public let `case`: String

	public init(default: String, case: String) {
		self.default = `default`
		self.case = `case`
	}
}

public enum `switch`: Codable {
	case `default`(`catch`)

	enum CodingKeys: String, CodingKey, Codable {
		case `default`
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .default:
				if let content = try? container.decode(`catch`.self, forKey: .content) {
					self = .default(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(`switch`.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for `switch`"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .default(let content):
			try container.encode(CodingKeys.default, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}

public enum `throws`: String, Codable {
	case `case`
	case `default`
}
