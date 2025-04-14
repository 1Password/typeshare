import Foundation

public struct OPItemDetailsFieldValue: Codable {
	public let hello: String

	public init(hello: String) {
		self.hello = hello
	}
}

public enum OPAdvancedColors: Codable {
	case string(String)
	case number(Int32)
	case numberArray([Int32])
	case reallyCoolType(OPItemDetailsFieldValue)
	case arrayReallyCoolType([OPItemDetailsFieldValue])
	case dictionaryReallyCoolType([String: OPItemDetailsFieldValue])

	enum CodingKeys: String, CodingKey, Codable {
		case string = "String",
			number = "Number",
			numberArray = "NumberArray",
			reallyCoolType = "ReallyCoolType",
			arrayReallyCoolType = "ArrayReallyCoolType",
			dictionaryReallyCoolType = "DictionaryReallyCoolType"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case t, c
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .t) {
			switch type {
			case .string:
				if let content = try? container.decode(String.self, forKey: .c) {
					self = .string(content)
					return
				}
			case .number:
				if let content = try? container.decode(Int32.self, forKey: .c) {
					self = .number(content)
					return
				}
			case .numberArray:
				if let content = try? container.decode([Int32].self, forKey: .c) {
					self = .numberArray(content)
					return
				}
			case .reallyCoolType:
				if let content = try? container.decode(OPItemDetailsFieldValue.self, forKey: .c) {
					self = .reallyCoolType(content)
					return
				}
			case .arrayReallyCoolType:
				if let content = try? container.decode([OPItemDetailsFieldValue].self, forKey: .c) {
					self = .arrayReallyCoolType(content)
					return
				}
			case .dictionaryReallyCoolType:
				if let content = try? container.decode([String: OPItemDetailsFieldValue].self, forKey: .c) {
					self = .dictionaryReallyCoolType(content)
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
			try container.encode(CodingKeys.string, forKey: .t)
			try container.encode(content, forKey: .c)
		case .number(let content):
			try container.encode(CodingKeys.number, forKey: .t)
			try container.encode(content, forKey: .c)
		case .numberArray(let content):
			try container.encode(CodingKeys.numberArray, forKey: .t)
			try container.encode(content, forKey: .c)
		case .reallyCoolType(let content):
			try container.encode(CodingKeys.reallyCoolType, forKey: .t)
			try container.encode(content, forKey: .c)
		case .arrayReallyCoolType(let content):
			try container.encode(CodingKeys.arrayReallyCoolType, forKey: .t)
			try container.encode(content, forKey: .c)
		case .dictionaryReallyCoolType(let content):
			try container.encode(CodingKeys.dictionaryReallyCoolType, forKey: .t)
			try container.encode(content, forKey: .c)
		}
	}
}
