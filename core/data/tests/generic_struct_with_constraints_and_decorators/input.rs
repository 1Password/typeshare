#[typeshare(
    swift = "Equatable, Identifiable",
    swiftGenericConstraints = "T: Equatable & SomeThingElse, V: Equatable"
)]
pub struct Button<T, V, I> {
    /// Label of the button
    pub label: I,
    /// Accessibility label if it needed to be different than label
    pub accessibility_label: Option<String>,
    /// Optional tooltips that provide extra explanation for a button
    pub tooltip: Option<String>,
    /// Button action if there one
    pub action: Option<T>,
    /// Icon if there is one
    pub icon: Option<V>,
    /// Button state
    pub state: ButtonState,
    /// Button Mode
    pub style: ButtonStyle,
}

#[typeshare]
pub struct ButtonState;

#[typeshare]
pub struct ButtonStyle;
