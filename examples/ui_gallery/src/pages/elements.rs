use gpui::{AnyElement, App, Window};
use ui::prelude::*;
use ui::{Avatar, Chip, Divider, Facepile};

use super::section;

/// Static "Elements" showcase: Buttons, Badges, Cards plus Phase 2's
/// ButtonGroup/Avatar/Facepile/Chip/Divider additions.
pub(crate) fn render(window: &mut Window, cx: &mut App) -> AnyElement {
    v_flex()
        .gap_8()
        .child(section("Buttons", Button::preview(window, cx)))
        .child(section("Button Groups", ButtonGroup::preview(window, cx)))
        .child(section("Badges", Badge::preview(window, cx)))
        .child(section("Cards", Card::preview(window, cx)))
        .child(section("Avatars", Avatar::preview(window, cx)))
        .child(section("Facepiles", Facepile::preview(window, cx)))
        .child(section("Chips", Chip::preview(window, cx)))
        .child(section("Dividers", Divider::preview(window, cx)))
        .into_any_element()
}
