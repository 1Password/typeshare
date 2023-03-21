import Foundation

public struct OPItemDetailsFieldValue: Codable {
	public init() {}
}

public enum OPAdvancedColors: Codable {
	case string(String)
	case number(Int32)
	case numberArray([Int32])
	case reallyCoolType(OPItemDetailsFieldValue)

	enum CodingKeys: String, CodingKey, Codable {
		case string,
			number,
			numberArray = "number-array",
			reallyCoolType
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .string:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .string(content)
					return
				}
			case .number:
				if let content = try? container.decode(Int32.self, forKey: .content) {
					self = .number(content)
					return
				}
			case .numberArray:
				if let content = try? container.decode([Int32].self, forKey: .content) {
					self = .numberArray(content)
					return
				}
			case .reallyCoolType:
				if let content = try? container.decode(OPItemDetailsFieldValue.self, forKey: .content) {
					self = .reallyCoolType(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OPAdvancedColors.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OPAdvancedColors"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .string(let content):
			try container.encode(CodingKeys.string, forKey: .type)
			try container.encode(content, forKey: .content)
		case .number(let content):
			try container.encode(CodingKeys.number, forKey: .type)
			try container.encode(content, forKey: .content)
		case .numberArray(let content):
			try container.encode(CodingKeys.numberArray, forKey: .type)
			try container.encode(content, forKey: .content)
		case .reallyCoolType(let content):
			try container.encode(CodingKeys.reallyCoolType, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
