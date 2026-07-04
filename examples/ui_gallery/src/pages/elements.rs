use gpui::{AnyElement, App, Window};
use ui::prelude::*;
use ui::{AspectRatio, Avatar, Chip, Divider, Facepile, Item, Kbd, Skeleton, Spinner, ToggleGroup};

use super::section;

/// Static "Elements" showcase: shadcn/ui core elements and aligned variants.
pub(crate) fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    let shadcn_buttons = h_flex()
        .flex_wrap()
        .gap_2()
        .child(
            Button::new("btn-default", "Default")
                .variant(ButtonVariant::Default)
                .shadcn_size(ButtonSizeAlias::Default),
        )
        .child(
            Button::new("btn-destructive", "Destructive")
                .variant(ButtonVariant::Destructive)
                .shadcn_size(ButtonSizeAlias::Default),
        )
        .child(
            Button::new("btn-outline", "Outline")
                .variant(ButtonVariant::Outline)
                .shadcn_size(ButtonSizeAlias::Default),
        )
        .child(
            Button::new("btn-secondary", "Secondary")
                .variant(ButtonVariant::Secondary)
                .shadcn_size(ButtonSizeAlias::Default),
        )
        .child(
            Button::new("btn-ghost", "Ghost")
                .variant(ButtonVariant::Ghost)
                .shadcn_size(ButtonSizeAlias::Default),
        )
        .child(
            Button::new("btn-sm", "Small")
                .variant(ButtonVariant::Default)
                .shadcn_size(ButtonSizeAlias::Sm),
        )
        .child(
            Button::new("btn-lg", "Large")
                .variant(ButtonVariant::Default)
                .shadcn_size(ButtonSizeAlias::Lg),
        );

    v_flex()
        .gap_8()
        .child(section("Buttons", Button::preview(window, cx)))
        .child(section(
            "Button Variants (shadcn)",
            Some(shadcn_buttons.into_any_element()),
        ))
        .child(section("Button Groups", ButtonGroup::preview(window, cx)))
        .child(section("Badges", Badge::preview(window, cx)))
        .child(section("Cards", Card::preview(window, cx)))
        .child(section("Avatars", Avatar::preview(window, cx)))
        .child(section("Facepiles", Facepile::preview(window, cx)))
        .child(section("Chips", Chip::preview(window, cx)))
        .child(section("Dividers", Divider::preview(window, cx)))
        .child(section("Skeleton", Skeleton::preview(window, cx)))
        .child(section("Aspect Ratio", AspectRatio::preview(window, cx)))
        .child(section("Kbd", Kbd::preview(window, cx)))
        .child(section("Spinner", Spinner::preview(window, cx)))
        .child(section("Checkbox", Checkbox::preview(window, cx)))
        .child(section("Switch", Switch::preview(window, cx)))
        .child(section("Toggle Group", ToggleGroup::preview(window, cx)))
        .child(section("Item", Item::preview(window, cx)))
        .child(section("Empty State", EmptyState::preview(window, cx)))
        .child(section("Form Field", FormField::preview(window, cx)))
        .into_any_element()
}
