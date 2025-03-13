import Foundation


/// Generated type representing the anonymous struct variant `Us` of the `AutofilledBy` Rust enum
public struct AutofilledByUsInner: Codable {
	/// The UUID for the fill
	public let uuid: String

	public init(uuid: String) {
		self.uuid = uuid
	}
}

/// Generated type representing the anonymous struct variant `SomethingElse` of the `AutofilledBy` Rust enum
public struct AutofilledBySomethingElseInner: Codable {
	/// The UUID for the fill
	public let uuid: String

	public init(uuid: String) {
		self.uuid = uuid
	}
}
/// Enum keeping track of who autofilled a field
public enum AutofilledBy: Codable {
	/// This field was autofilled by us
	case us(AutofilledByUsInner)
	/// Something else autofilled this field
	case somethingElse(AutofilledBySomethingElseInner)

	enum CodingKeys: String, CodingKey, Codable {
		case us = "Us",
			somethingElse = "SomethingElse"
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .us:
				if let content = try? container.decode(AutofilledByUsInner.self, forKey: .content) {
					self = .us(content)
					return
				}
			case .somethingElse:
				if let content = try? container.decode(AutofilledBySomethingElseInner.self, forKey: .content) {
					self = .somethingElse(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(AutofilledBy.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for AutofilledBy"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .us(let content):
			try container.encode(CodingKeys.us, forKey: .type)
			try container.encode(content, forKey: .content)
		case .somethingElse(let content):
			try container.encode(CodingKeys.somethingElse, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
