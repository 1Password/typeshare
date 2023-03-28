import Foundation

public struct Foo: Codable {
	public let url: String

	public init(url: String) {
		self.url = url
	}
}
