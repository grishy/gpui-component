use crate::{
    ActiveTheme, Disableable, FocusableExt, Side, Sizable, Size, StyledExt, h_flex, text::Text,
    tooltip::ComponentTooltip,
};
use gpui::{
    Animation, AnimationExt as _, App, Background, ElementId, Hsla, InteractiveElement,
    IntoElement, ParentElement as _, RenderOnce, Role, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Toggled, Window, div, prelude::FluentBuilder as _, px,
};
use std::{rc::Rc, time::Duration};

/// A Switch element that can be toggled on or off.
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    style: StyleRefinement,
    checked: bool,
    disabled: bool,
    label: Option<Text>,
    aria_label: Option<SharedString>,
    label_side: Side,
    on_click: Option<Rc<dyn Fn(&bool, &mut Window, &mut App)>>,
    size: Size,
    color: Option<Hsla>,
    tooltip: ComponentTooltip,
}

impl Switch {
    /// Create a new Switch element.
    pub fn new(id: impl Into<ElementId>) -> Self {
        let id: ElementId = id.into();
        Self {
            id: id.clone(),
            style: StyleRefinement::default(),
            checked: false,
            disabled: false,
            label: None,
            aria_label: None,
            on_click: None,
            label_side: Side::Right,
            size: Size::Medium,
            color: None,
            tooltip: ComponentTooltip::default(),
        }
    }

    /// Set the checked state of the switch.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the label of the switch.
    pub fn label(mut self, label: impl Into<Text>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the label announced by assistive technology.
    ///
    /// This overrides the visible label for accessibility naming.
    pub fn aria_label(mut self, label: impl Into<SharedString>) -> Self {
        self.aria_label = Some(label.into());
        self
    }

    fn a11y_label(&self, cx: &App) -> Option<SharedString> {
        self.aria_label
            .clone()
            .or_else(|| self.label.as_ref().map(|label| label.get_text(cx)))
    }

    fn a11y_toggled(&self) -> Toggled {
        if self.checked {
            Toggled::True
        } else {
            Toggled::False
        }
    }

    /// Add a click handler for the switch.
    pub fn on_click<F>(mut self, handler: F) -> Self
    where
        F: Fn(&bool, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Rc::new(handler));
        self
    }

    /// Set the background color of the switch when checked.
    /// Defaults to `cx.theme().primary`.
    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Set tooltip text for the switch.
    pub fn tooltip(mut self, tooltip: impl Into<SharedString>) -> Self {
        self.tooltip.text = Some((tooltip.into(), None));
        self
    }
}

impl Styled for Switch {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        &mut self.style
    }
}

impl Sizable for Switch {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Disableable for Switch {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let on_click = self.on_click.clone();
        let a11y_label = self.a11y_label(cx);
        let a11y_toggled = self.a11y_toggled();
        let focus_handle = window
            .use_keyed_state(self.id.clone(), cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let show_focus_ring = !self.disabled && focus_handle.is_focused(window);
        let toggle_state = window.use_keyed_state(self.id.clone(), cx, |_, _| checked);

        let checked_bg = self
            .color
            .map(Background::from)
            .unwrap_or(cx.theme().tokens.primary.into());
        let (bg, toggle_bg): (Background, Background) = match checked {
            true => (checked_bg, cx.theme().tokens.switch_thumb.into()),
            false => (
                cx.theme().tokens.switch.into(),
                cx.theme().tokens.switch_thumb.into(),
            ),
        };

        let (bg, toggle_bg) = if self.disabled {
            (
                if checked { bg.opacity(0.5) } else { bg },
                toggle_bg.opacity(0.35),
            )
        } else {
            (bg, toggle_bg)
        };

        let (bg_width, bg_height) = match self.size {
            Size::XSmall | Size::Small => (px(28.), px(16.)),
            _ => (px(36.), px(20.)),
        };
        let bar_width = match self.size {
            Size::XSmall | Size::Small => px(12.),
            _ => px(16.),
        };
        let inset = px(2.);
        let radius = if cx.theme().radius >= px(4.) {
            bg_height
        } else {
            cx.theme().radius
        };

        div().refine_style(&self.style).child(
            h_flex()
                .id(self.id.clone())
                .role(Role::Switch)
                .aria_toggled(a11y_toggled)
                .when_some(a11y_label, |this, label| this.aria_label(label))
                .when(!self.disabled, |this| {
                    this.track_focus(&focus_handle.tab_stop(true))
                })
                .gap_2()
                .items_start()
                .when(self.label_side.is_left(), |this| this.flex_row_reverse())
                .rounded(cx.theme().radius * 0.5)
                .focus_ring(show_focus_ring, px(2.), window, cx)
                .child(
                    // Switch Bar
                    div()
                        .w(bg_width)
                        .h(bg_height)
                        .rounded(radius)
                        .flex()
                        .items_center()
                        .border(inset)
                        .border_color(cx.theme().transparent)
                        .bg(bg)
                        .child(
                            // Switch Toggle
                            div()
                                .rounded(radius)
                                .bg(toggle_bg)
                                .shadow_md()
                                .size(bar_width)
                                .map(|this| {
                                    let prev_checked = toggle_state.read(cx);
                                    if !self.disabled && *prev_checked != checked {
                                        let duration = Duration::from_secs_f64(0.15);
                                        cx.spawn({
                                            let toggle_state = toggle_state.clone();
                                            async move |cx| {
                                                cx.background_executor().timer(duration).await;
                                                _ = toggle_state
                                                    .update(cx, |this, _| *this = checked);
                                            }
                                        })
                                        .detach();

                                        this.with_animation(
                                            ElementId::NamedInteger("move".into(), checked as u64),
                                            Animation::new(duration),
                                            move |this, delta| {
                                                let max_x = bg_width - bar_width - inset * 2;
                                                let x = if checked {
                                                    max_x * delta
                                                } else {
                                                    max_x - max_x * delta
                                                };
                                                this.left(x)
                                            },
                                        )
                                        .into_any_element()
                                    } else {
                                        let max_x = bg_width - bar_width - inset * 2;
                                        let x = if checked { max_x } else { px(0.) };
                                        this.left(x).into_any_element()
                                    }
                                }),
                        ),
                )
                .when_some(self.label, |this, label| {
                    this.child(div().line_height(bg_height).child(label).map(
                        |this| match self.size {
                            Size::XSmall | Size::Small => this.text_sm(),
                            _ => this.text_base(),
                        },
                    ))
                })
                .on_mouse_down(gpui::MouseButton::Left, |_, window, _| {
                    // Avoid focus on mouse down.
                    window.prevent_default();
                })
                .when_some(
                    on_click
                        .as_ref()
                        .map(|c| c.clone())
                        .filter(|_| !self.disabled),
                    |this, on_click| {
                        let toggle_state = toggle_state.clone();
                        this.on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            _ = toggle_state.update(cx, |this, _| *this = checked);
                            on_click(&!checked, window, cx);
                        })
                    },
                )
                .map(|this| self.tooltip.apply(this)),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Toggled;

    #[gpui::test]
    fn a11y_name_and_state_follow_switch_configuration(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let unchecked = Switch::new("visible-label").label("Visible label");
            assert_eq!(unchecked.a11y_label(cx), Some("Visible label".into()));
            assert_eq!(unchecked.a11y_toggled(), Toggled::False);

            let checked = Switch::new("explicit-label")
                .label("Visible label")
                .aria_label("Explicit label")
                .checked(true);
            assert_eq!(checked.a11y_label(cx), Some("Explicit label".into()));
            assert_eq!(checked.a11y_toggled(), Toggled::True);
        });
    }

    #[gpui::test]
    fn tooltip_is_not_an_accessible_name(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            let switch = Switch::new("tooltip-only").tooltip("Visual hint");

            assert_eq!(switch.a11y_label(cx), None);
        });
    }
}
