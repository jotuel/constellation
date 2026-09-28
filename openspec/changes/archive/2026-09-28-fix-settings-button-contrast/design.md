# Design

## Context

Settings navigation overview rows (`category_row`) and header cards (`header_card`) in `src/settings/widgets/navigation.rs` are styled using `.class(cosmic::theme::Button::ListItem(radii))`. In libcosmic's button stylesheet, `Button::ListItem` explicitly assigns `appearance.text_color` and `appearance.icon_color` from `cosmic.list_button.on`.

In light theme mode, `cosmic.list_button.on` resolves to a near-white/light-grey color (`srgba(222, 222, 222, 1)`) against a light drawer surface (`srgba(235, 235, 235, 1)`), resulting in severe contrast loss where text and icons appear disabled.

## Goals / Non-Goals

**Goals:**
- Provide a dedicated button style function in `navigation.rs` returning `cosmic::theme::Button::Custom`.
- Leave `text_color` and `icon_color` as `None` across active, hovered, and pressed states so child text and icon elements naturally inherit the high-contrast contextual text color from the renderer.
- Retain identical visual metrics: theme corner radii, padding, and native list-button hover/pressed overlay feedback.
- Apply this consistent styling to both `category_row` and `header_card`.

**Non-Goals:**
- Patching or modifying upstream `libcosmic` or `cosmic-theme` crates.
- Changing settings routing, message passing, or subpage navigation stacks.
- Redesigning settings layout into boxed card containers.

## Decisions

### Decision: Implement `list_item_button_style` via `Button::Custom`
- Implement a helper function `list_item_button_style(radii: [f32; 4]) -> cosmic::theme::Button` in `navigation.rs`.
- Define the state closures:
  - **`active`**: Transparent background (`None`), `text_color: None`, `icon_color: None`, corner radius matching theme `radii`.
  - **`hovered`**: Background `Some(Background::Color(cosmic.list_button.hover.into()))`, `text_color: None`, `icon_color: None`.
  - **`pressed`**: Background `Some(Background::Color(cosmic.list_button.pressed.into()))`, `text_color: None`, `icon_color: None`.
  - **`disabled`**: Background `None`, `text_color: Some(cosmic.list_button.on_disabled.into())`, `icon_color: Some(cosmic.list_button.on_disabled.into())`.
- Focus outlines use standard accent color if focused.

**Alternatives considered:**
- *Using `Button::Transparent`*: While it leaves text color unconstrained, it lacks list-button hover/pressed feedback and corner radius control.
- *Wrapping rows in `Container::List`*: Introduces blocky card containers that deviate from the borderless list overview design.

## Risks / Trade-offs

- **Closure Allocation**: `Button::Custom` requires boxed closures (`Box<dyn Fn...>`). Since settings overview pages contain fewer than 10 category rows and are only rendered when the settings drawer is visible, allocation overhead is negligible.
