#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DisplayType {
    Block,
    Inline,
    None,
    Flex,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unit {
    Px(f32),
    Pct(f32),
    Auto,
    Zero,
}
