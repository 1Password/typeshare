import Foundation

public struct Button<T: Codable & Equatable>: Codable, Equatable {
	/// Label of the button
	public let label: String
	/// Accessibility label if it needed to be different than label
	public let accessibility_label: String?
	/// Optional tooltips that provide extra explanation for a button
	public let tooltip: String?
	/// Button action if there one
	public let action: T?
	/// Icon if there is one
	public let icon: Icon?
	/// Button state
	public let state: ButtonState
	/// Button Mode
	public let style: ButtonStyle

	public init(label: String, accessibility_label: String?, tooltip: String?, action: T?, icon: Icon?, state: ButtonState, style: ButtonStyle) {
		self.label = label
		self.accessibility_label = accessibility_label
		self.tooltip = tooltip
		self.action = action
		self.icon = icon
		self.state = state
		self.style = style
	}
}
