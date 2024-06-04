#[typeshare(swift = "Equatable")]
pub struct Button<T> {
    /// Label of the button
    pub label: String,
    /// Accessibility label if it needed to be different than label
    pub accessibility_label: Option<String>,
    /// Optional tooltips that provide extra explanation for a button
    pub tooltip: Option<String>,
    /// Button action if there one
    pub action: Option<T>,
    /// Icon if there is one
    pub icon: Option<Icon>,
    /// Button state
    pub state: ButtonState,
    /// Button Mode
    pub style: ButtonStyle,
}
