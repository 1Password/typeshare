import Foundation

public struct OverrideStruct: Codable {
	public let fieldToOverride: Int

	public init(fieldToOverride: Int) {
		self.fieldToOverride = fieldToOverride
	}
}


/// Generated type representing the anonymous struct variant `AnonymousStructVariant` of the `OverrideEnum` Rust enum
public struct OverrideEnumAnonymousStructVariantInner: Codable {
	public let fieldToOverride: Int

	public init(fieldToOverride: Int) {
		self.fieldToOverride = fieldToOverride
	}
}
public enum OverrideEnum: Codable {
	case unitVariant
	case tupleVariant(String)
	case anonymousStructVariant(OverrideEnumAnonymousStructVariantInner)

	enum CodingKeys: String, CodingKey, Codable {
		case unitVariant = "UnitVariant",
			tupleVariant = "TupleVariant",
			anonymousStructVariant = "AnonymousStructVariant"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .unitVariant:
				self = .unitVariant
				return
			case .tupleVariant:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .tupleVariant(content)
					return
				}
			case .anonymousStructVariant:
				if let content = try? container.decode(OverrideEnumAnonymousStructVariantInner.self, forKey: .content) {
					self = .anonymousStructVariant(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(OverrideEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for OverrideEnum"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .unitVariant:
			try container.encode(CodingKeys.unitVariant, forKey: .type)
		case .tupleVariant(let content):
			try container.encode(CodingKeys.tupleVariant, forKey: .type)
			try container.encode(content, forKey: .content)
		case .anonymousStructVariant(let content):
			try container.encode(CodingKeys.anonymousStructVariant, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
