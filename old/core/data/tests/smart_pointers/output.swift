import Foundation

/// This is a comment.
public struct ArcyColors: Codable {
	public let red: UInt8
	public let blue: String
	public let green: [String]

	public init(red: UInt8, blue: String, green: [String]) {
		self.red = red
		self.blue = blue
		self.green = green
	}
}

/// This is a comment.
public struct CellyColors: Codable {
	public let red: String
	public let blue: [String]

	public init(red: String, blue: [String]) {
		self.red = red
		self.blue = blue
	}
}

/// This is a comment.
public struct CowyColors: Codable {
	public let lifetime: String

	public init(lifetime: String) {
		self.lifetime = lifetime
	}
}

/// This is a comment.
public struct LockyColors: Codable {
	public let red: String

	public init(red: String) {
		self.red = red
	}
}

/// This is a comment.
public struct MutexyColors: Codable {
	public let blue: [String]
	public let green: String

	public init(blue: [String], green: String) {
		self.blue = blue
		self.green = green
	}
}

/// This is a comment.
public struct RcyColors: Codable {
	public let red: String
	public let blue: [String]
	public let green: String

	public init(red: String, blue: [String], green: String) {
		self.red = red
		self.blue = blue
		self.green = green
	}
}

/// This is a comment.
public enum BoxyColors: Codable {
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
		throw DecodingError.typeMismatch(BoxyColors.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for BoxyColors"))
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
