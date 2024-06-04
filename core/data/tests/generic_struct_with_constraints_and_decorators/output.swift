import Foundation

public struct ButtonState: Codable {
	public init() {}
}

public struct ButtonStyle: Codable {
	public init() {}
}

public struct Button<T: Codable & Equatable & SomeThingElse, V: Codable & Equatable, I: Codable>: Codable, Equatable, Identifiable {
	/// Label of the button
	public let label: I
	/// Accessibility label if it needed to be different than label
	public let accessibility_label: String?
	/// Optional tooltips that provide extra explanation for a button
	public let tooltip: String?
	/// Button action if there one
	public let action: T?
	/// Icon if there is one
	public let icon: V?
	/// Button state
	public let state: ButtonState
	/// Button Mode
	public let style: ButtonStyle

	public init(label: I, accessibility_label: String?, tooltip: String?, action: T?, icon: V?, state: ButtonState, style: ButtonStyle) {
		self.label = label
		self.accessibility_label = accessibility_label
		self.tooltip = tooltip
		self.action = action
		self.icon = icon
		self.state = state
		self.style = style
	}
}
