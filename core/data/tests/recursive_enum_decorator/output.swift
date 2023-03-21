import Foundation

public indirect enum Options: Codable {
	case red(Bool)
	case banana(String)
	case vermont(Options)

	enum CodingKeys: String, CodingKey, Codable {
		case red,
			banana,
			vermont
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .red:
				if let content = try? container.decode(Bool.self, forKey: .content) {
					self = .red(content)
					return
				}
			case .banana:
				if let content = try? container.decode(String.self, forKey: .content) {
					self = .banana(content)
					return
				}
			case .vermont:
				if let content = try? container.decode(Options.self, forKey: .content) {
					self = .vermont(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(Options.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for Options"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .red(let content):
			try container.encode(CodingKeys.red, forKey: .type)
			try container.encode(content, forKey: .content)
		case .banana(let content):
			try container.encode(CodingKeys.banana, forKey: .type)
			try container.encode(content, forKey: .content)
		case .vermont(let content):
			try container.encode(CodingKeys.vermont, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}


/// Generated type representing the anonymous struct variant `Exactly` of the `MoreOptions` Rust enum
public struct MoreOptionsExactlyInner: Codable {
	public let config: String

	public init(config: String) {
		self.config = config
	}
}

/// Generated type representing the anonymous struct variant `Built` of the `MoreOptions` Rust enum
public struct MoreOptionsBuiltInner: Codable {
	public let top: MoreOptions

	public init(top: MoreOptions) {
		self.top = top
	}
}
public indirect enum MoreOptions: Codable {
	case news(Bool)
	case exactly(MoreOptionsExactlyInner)
	case built(MoreOptionsBuiltInner)

	enum CodingKeys: String, CodingKey, Codable {
		case news,
			exactly,
			built
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .news:
				if let content = try? container.decode(Bool.self, forKey: .content) {
					self = .news(content)
					return
				}
			case .exactly:
				if let content = try? container.decode(MoreOptionsExactlyInner.self, forKey: .content) {
					self = .exactly(content)
					return
				}
			case .built:
				if let content = try? container.decode(MoreOptionsBuiltInner.self, forKey: .content) {
					self = .built(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(MoreOptions.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for MoreOptions"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .news(let content):
			try container.encode(CodingKeys.news, forKey: .type)
			try container.encode(content, forKey: .content)
		case .exactly(let content):
			try container.encode(CodingKeys.exactly, forKey: .type)
			try container.encode(content, forKey: .content)
		case .built(let content):
			try container.encode(CodingKeys.built, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
