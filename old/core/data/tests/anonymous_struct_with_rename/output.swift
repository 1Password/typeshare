import Foundation


/// Generated type representing the anonymous struct variant `List` of the `AnonymousStructWithRename` Rust enum
public struct CoreAnonymousStructWithRenameListInner: Codable {
	public let list: [String]

	public init(list: [String]) {
		self.list = list
	}
}

/// Generated type representing the anonymous struct variant `LongFieldNames` of the `AnonymousStructWithRename` Rust enum
public struct CoreAnonymousStructWithRenameLongFieldNamesInner: Codable {
	public let some_long_field_name: String
	public let and: Bool
	public let but_one_more: [String]

	public init(some_long_field_name: String, and: Bool, but_one_more: [String]) {
		self.some_long_field_name = some_long_field_name
		self.and = and
		self.but_one_more = but_one_more
	}
}

/// Generated type representing the anonymous struct variant `KebabCase` of the `AnonymousStructWithRename` Rust enum
public struct CoreAnonymousStructWithRenameKebabCaseInner: Codable {
	public let another_list: [String]
	public let camelCaseStringField: String
	public let something_else: Bool

	enum CodingKeys: String, CodingKey, Codable {
		case another_list = "another-list",
			camelCaseStringField,
			something_else = "something-else"
	}

	public init(another_list: [String], camelCaseStringField: String, something_else: Bool) {
		self.another_list = another_list
		self.camelCaseStringField = camelCaseStringField
		self.something_else = something_else
	}
}
public enum CoreAnonymousStructWithRename: Codable {
	case list(CoreAnonymousStructWithRenameListInner)
	case longFieldNames(CoreAnonymousStructWithRenameLongFieldNamesInner)
	case kebabCase(CoreAnonymousStructWithRenameKebabCaseInner)

	enum CodingKeys: String, CodingKey, Codable {
		case list,
			longFieldNames,
			kebabCase
	}

	private enum ContainerCodingKeys: String, CodingKey {
		case type, content
	}

	public init(from decoder: Decoder) throws {
		let container = try decoder.container(keyedBy: ContainerCodingKeys.self)
		if let type = try? container.decode(CodingKeys.self, forKey: .type) {
			switch type {
			case .list:
				if let content = try? container.decode(CoreAnonymousStructWithRenameListInner.self, forKey: .content) {
					self = .list(content)
					return
				}
			case .longFieldNames:
				if let content = try? container.decode(CoreAnonymousStructWithRenameLongFieldNamesInner.self, forKey: .content) {
					self = .longFieldNames(content)
					return
				}
			case .kebabCase:
				if let content = try? container.decode(CoreAnonymousStructWithRenameKebabCaseInner.self, forKey: .content) {
					self = .kebabCase(content)
					return
				}
			}
		}
		throw DecodingError.typeMismatch(CoreAnonymousStructWithRename.self, DecodingError.Context(codingPath: decoder.codingPath, debugDescription: "Wrong type for CoreAnonymousStructWithRename"))
	}

	public func encode(to encoder: Encoder) throws {
		var container = encoder.container(keyedBy: ContainerCodingKeys.self)
		switch self {
		case .list(let content):
			try container.encode(CodingKeys.list, forKey: .type)
			try container.encode(content, forKey: .content)
		case .longFieldNames(let content):
			try container.encode(CodingKeys.longFieldNames, forKey: .type)
			try container.encode(content, forKey: .content)
		case .kebabCase(let content):
			try container.encode(CodingKeys.kebabCase, forKey: .type)
			try container.encode(content, forKey: .content)
		}
	}
}
