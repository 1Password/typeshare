import Foundation

public struct GenericType<K: Codable & Identifiable & Sendable, V: Codable & Identifiable & Sendable>: Codable {
	public let key: K
	public let value: V

	public init(key: K, value: V) {
		self.key = key
		self.value = value
	}
}


/// Generated type representing the anonymous struct variant `Variant` of the `GenericEnum` Rust enum
public struct GenericEnumVariantInner<K: Codable & Identifiable & Sendable, V: Codable & Identifiable & Sendable>: Codable {
	public let key: K
	public let value: V

	public init(key: K, value: V) {
		self.key = key
		self.value = value
	}
}
public enum GenericEnum<K: Codable & Identifiable & Sendable, V: Codable & Identifiable & Sendable>: Codable {
	case variant(GenericEnumVariantInner<K, V>)

	enum CodingKeys: String, CodingKey, Codable {
		case variant = "Variant"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .variant:
				if let content = try? container.decode(GenericEnumVariantInner<K, V>.self, forKey: .content) {
					self = .variant(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(GenericEnum.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for GenericEnum"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .variant(let content):
			try container.encode(CodingKeys.variant, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
