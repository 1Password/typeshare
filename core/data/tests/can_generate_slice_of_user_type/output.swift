import Foundation

public struct Video: Codable {
	public let tags: [Tag]

	public init(tags: [Tag]) {
		self.tags = tags
	}
}
