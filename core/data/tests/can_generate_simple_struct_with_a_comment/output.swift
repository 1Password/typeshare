import Foundation

public struct Location: Codable {
	public init() {}
}

/// This is a comment.
public struct Person: Codable {
	/// This is another comment
	public let name: String
	public let age: UInt8
	public let info: String?
	public let emails: [String]
	public let location: Location

	public init(name: String, age: UInt8, info: String?, emails: [String], location: Location) {
		self.name = name
		self.age = age
		self.info = info
		self.emails = emails
		self.location = location
	}
}
