import Foundation

public struct Foo: Codable {
	public let time: Date

	public init(time: Date) {
		self.time = time
	}
}
