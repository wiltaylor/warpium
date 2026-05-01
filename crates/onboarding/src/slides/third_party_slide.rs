use super::toggle_card::{render_toggle_card, ToggleCardSpec};
use super::OnboardingSlide;
use crate::model::{OnboardingStateEvent, OnboardingStateModel};
use crate::slides::{bottom_nav, layout, slide_content};
use crate::OnboardingIntention;

use ui_components::{button, Component as _, Options as _};
use warp_core::ui::appearance::Appearance;
use warp_core::ui::icons::Icon;
use warp_core::ui::theme::{color::internal_colors, Fill};
use warpui::prelude::Align;
use warpui::{
    elements::{
        Border, ClippedScrollStateHandle, ConstrainedBox, Container, CornerRadius,
        CrossAxisAlignment, Flex, FormattedTextElement, Hoverable, Icon as WarpUiIcon,
        MainAxisAlignment, MainAxisSize, MouseStateHandle, ParentElement, Radius, Text,
    },
    fonts::{Properties, Weight},
    keymap::Keystroke,
    platform::Cursor,
    text_layout::TextAlignment,
    ui_components::components::{UiComponent as _, UiComponentStyles},
    AppContext, Element, Entity, ModelHandle, SingletonEntity as _, TypedActionView, View,
    ViewContext,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThirdPartyAgentHandler {
    ClaudeCode,
    #[default]
    Codex,
    Gemini,
}

impl ThirdPartyAgentHandler {
    pub const ALL: [Self; 3] = [Self::ClaudeCode, Self::Codex, Self::Gemini];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude Code",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini",
        }
    }

    pub fn serialized_name(self) -> &'static str {
        match self {
            Self::ClaudeCode => "Claude",
            Self::Codex => "Codex",
            Self::Gemini => "Gemini",
        }
    }

    fn icon(self) -> Icon {
        match self {
            Self::ClaudeCode => Icon::ClaudeLogo,
            Self::Codex => Icon::OpenAILogo,
            Self::Gemini => Icon::GeminiLogo,
        }
    }

    fn previous(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|agent| *agent == self)
            .unwrap_or_default();
        let previous = if index == 0 {
            Self::ALL.len() - 1
        } else {
            index - 1
        };
        Self::ALL[previous]
    }

    fn next(self) -> Self {
        let index = Self::ALL
            .iter()
            .position(|agent| *agent == self)
            .unwrap_or_default();
        Self::ALL[(index + 1) % Self::ALL.len()]
    }
}

/// Which setting card is currently expanded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingCard {
    AgentCommand,
    CliToolbar,
    Notifications,
}

#[derive(Debug, Clone)]
pub enum ThirdPartySlideAction {
    SelectSettingCard { card: SettingCard },
    SelectAgentCommandHandler { agent: ThirdPartyAgentHandler },
    SetCliAgentToolbarEnabled { enabled: bool },
    SetShowAgentNotifications { enabled: bool },
    BackClicked,
    NextClicked,
}

pub struct ThirdPartySlide {
    onboarding_state: ModelHandle<OnboardingStateModel>,
    selected_setting: Option<SettingCard>,
    agent_command_card_mouse_state: MouseStateHandle,
    agent_command_option_mouse_states: Vec<MouseStateHandle>,
    cli_toolbar_card_mouse_state: MouseStateHandle,
    notifications_card_mouse_state: MouseStateHandle,
    cli_toolbar_seg_left_mouse: MouseStateHandle,
    cli_toolbar_seg_right_mouse: MouseStateHandle,
    notifications_seg_left_mouse: MouseStateHandle,
    notifications_seg_right_mouse: MouseStateHandle,
    back_button: button::Button,
    next_button: button::Button,
    scroll_state: ClippedScrollStateHandle,
}

impl ThirdPartySlide {
    pub(crate) fn new(
        onboarding_state: ModelHandle<OnboardingStateModel>,
        ctx: &mut ViewContext<Self>,
    ) -> Self {
        ctx.subscribe_to_model(&onboarding_state, |_me, _model, event, ctx| {
            if matches!(event, OnboardingStateEvent::IntentionChanged) {
                ctx.notify();
            }
        });

        Self {
            onboarding_state,
            selected_setting: None,
            agent_command_card_mouse_state: MouseStateHandle::default(),
            agent_command_option_mouse_states: ThirdPartyAgentHandler::ALL
                .iter()
                .map(|_| MouseStateHandle::default())
                .collect(),
            cli_toolbar_card_mouse_state: MouseStateHandle::default(),
            notifications_card_mouse_state: MouseStateHandle::default(),
            cli_toolbar_seg_left_mouse: MouseStateHandle::default(),
            cli_toolbar_seg_right_mouse: MouseStateHandle::default(),
            notifications_seg_left_mouse: MouseStateHandle::default(),
            notifications_seg_right_mouse: MouseStateHandle::default(),
            back_button: button::Button::default(),
            next_button: button::Button::default(),
            scroll_state: ClippedScrollStateHandle::new(),
        }
    }

    /// All onboarding image paths used by this slide's visual.
    pub(crate) const VISUAL_IMAGE_PATHS: &'static [&'static str] = &[
        "async/png/onboarding/thirdparty_toolbar_enabled_vertical.png",
        "async/png/onboarding/thirdparty_toolbar_enabled_horizontal.png",
        "async/png/onboarding/thirdparty_toolbar_disabled_vertical.png",
        "async/png/onboarding/thirdparty_toolbar_disabled_horizontal.png",
        "async/png/onboarding/thirdparty_notifications_enabled.png",
        "async/png/onboarding/thirdparty_notifications_disabled.png",
    ];

    fn cli_agent_toolbar_enabled(&self, app: &AppContext) -> bool {
        self.onboarding_state
            .as_ref(app)
            .agent_settings()
            .cli_agent_toolbar_enabled
    }

    fn show_agent_notifications(&self, app: &AppContext) -> bool {
        self.onboarding_state
            .as_ref(app)
            .agent_settings()
            .show_agent_notifications
    }

    fn agent_command_handler(&self, app: &AppContext) -> ThirdPartyAgentHandler {
        self.onboarding_state
            .as_ref(app)
            .agent_settings()
            .agent_command_handler
    }

    fn model_intention(&self, app: &AppContext) -> OnboardingIntention {
        *self.onboarding_state.as_ref(app).intention()
    }

    fn render_content(
        &self,
        appearance: &Appearance,
        agent_command_handler: ThirdPartyAgentHandler,
        cli_toolbar_enabled: bool,
        show_agent_notifications: bool,
        intention: OnboardingIntention,
    ) -> Box<dyn Element> {
        let bottom_nav = Align::new(self.render_bottom_nav(appearance, intention)).finish();

        let mut sections = vec![
            self.render_header(appearance),
            self.render_agent_handler_section(appearance, agent_command_handler),
            self.render_toolbar_section(appearance, cli_toolbar_enabled),
        ];

        // Only show the notifications toggle for terminal intention.
        // For agent intention, notifications are always enabled.
        if matches!(intention, OnboardingIntention::Terminal) {
            sections.push(self.render_notifications_section(appearance, show_agent_notifications));
        }

        slide_content::onboarding_slide_content(
            sections,
            bottom_nav,
            self.scroll_state.clone(),
            appearance,
        )
    }

    fn render_header(&self, appearance: &Appearance) -> Box<dyn Element> {
        let title = appearance
            .ui_builder()
            .paragraph("Customize third party agents")
            .with_style(UiComponentStyles {
                font_size: Some(36.),
                font_weight: Some(Weight::Medium),
                ..Default::default()
            })
            .build()
            .finish();

        let subtitle = FormattedTextElement::from_str(
            "Select defaults for using agents like Claude Code, Codex, and Gemini.",
            appearance.ui_font_family(),
            16.,
        )
        .with_color(internal_colors::text_sub(
            appearance.theme(),
            appearance.theme().background().into_solid(),
        ))
        .with_weight(Weight::Normal)
        .with_alignment(TextAlignment::Left)
        .with_line_height_ratio(1.0)
        .finish();

        Flex::column()
            .with_main_axis_size(MainAxisSize::Min)
            .with_cross_axis_alignment(CrossAxisAlignment::Start)
            .with_child(title)
            .with_child(Container::new(subtitle).with_margin_top(16.).finish())
            .finish()
    }

    fn render_agent_handler_section(
        &self,
        appearance: &Appearance,
        selected_agent: ThirdPartyAgentHandler,
    ) -> Box<dyn Element> {
        let is_selected = self.selected_setting == Some(SettingCard::AgentCommand);
        let card = if is_selected {
            self.render_agent_handler_expanded(appearance, selected_agent)
        } else {
            self.render_agent_handler_collapsed(appearance, selected_agent)
        };

        Container::new(
            Flex::column()
                .with_main_axis_size(MainAxisSize::Min)
                .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
                .with_child(card)
                .finish(),
        )
        .with_margin_top(40.)
        .finish()
    }

    fn render_agent_handler_collapsed(
        &self,
        appearance: &Appearance,
        selected_agent: ThirdPartyAgentHandler,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let ui_font_family = appearance.ui_font_family();
        let text_color = internal_colors::text_sub(theme, theme.background().into_solid());
        let border_color = Fill::Solid(internal_colors::neutral_4(theme));
        let subtitle = selected_agent.display_name().to_string();

        Hoverable::new(self.agent_command_card_mouse_state.clone(), move |_| {
            let title_el =
                FormattedTextElement::from_str("Browser /agent handler", ui_font_family, 16.)
                    .with_color(text_color)
                    .with_weight(Weight::Normal)
                    .with_alignment(TextAlignment::Left)
                    .with_line_height_ratio(1.0)
                    .finish();

            let sub_el = Text::new(subtitle.clone(), ui_font_family, 12.)
                .with_color(text_color)
                .with_line_height_ratio(1.0)
                .finish();

            let content = Flex::column()
                .with_main_axis_size(MainAxisSize::Min)
                .with_cross_axis_alignment(CrossAxisAlignment::Start)
                .with_child(title_el)
                .with_child(Container::new(sub_el).with_margin_top(12.).finish())
                .finish();

            Container::new(content)
                .with_uniform_padding(24.)
                .with_corner_radius(CornerRadius::with_all(Radius::Pixels(8.)))
                .with_border(Border::all(1.).with_border_fill(border_color))
                .finish()
        })
        .with_cursor(Cursor::PointingHand)
        .on_click(|ctx, _, _| {
            ctx.dispatch_typed_action(ThirdPartySlideAction::SelectSettingCard {
                card: SettingCard::AgentCommand,
            });
        })
        .finish()
    }

    fn render_agent_handler_expanded(
        &self,
        appearance: &Appearance,
        selected_agent: ThirdPartyAgentHandler,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let ui_font_family = appearance.ui_font_family();
        let text_color = internal_colors::text_main(theme, theme.background().into_solid());
        let border_color = theme.accent();
        let background = internal_colors::accent_overlay_1(theme);

        let title_el =
            FormattedTextElement::from_str("Browser /agent handler", ui_font_family, 16.)
                .with_color(text_color)
                .with_weight(Weight::Normal)
                .with_alignment(TextAlignment::Left)
                .with_line_height_ratio(1.0)
                .finish();

        let selected_control = self.render_agent_handler_select(appearance, selected_agent);
        let options = self.render_agent_handler_options(appearance, selected_agent);

        Container::new(
            Flex::column()
                .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
                .with_child(title_el)
                .with_child(
                    Container::new(selected_control)
                        .with_margin_top(12.)
                        .finish(),
                )
                .with_child(Container::new(options).with_margin_top(8.).finish())
                .finish(),
        )
        .with_uniform_padding(24.)
        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(8.)))
        .with_border(Border::all(1.).with_border_fill(border_color))
        .with_background(background)
        .finish()
    }

    fn render_agent_handler_select(
        &self,
        appearance: &Appearance,
        selected_agent: ThirdPartyAgentHandler,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let color = internal_colors::text_main(theme, theme.background().into_solid());
        let icon = selected_agent.icon();
        let label = selected_agent.display_name().to_string();
        let ui_font_family = appearance.ui_font_family();

        let icon_el = ConstrainedBox::new(Box::new(icon.to_warpui_icon(color.into())))
            .with_width(14.)
            .with_height(14.)
            .finish();
        let label_el = Text::new(label, ui_font_family, 14.)
            .with_color(color)
            .with_style(Properties {
                weight: Weight::Normal,
                ..Default::default()
            })
            .with_line_height_ratio(1.0)
            .finish();
        let chevron = ConstrainedBox::new(Box::new(WarpUiIcon::new(
            "bundled/svg/chevron-down.svg",
            color,
        )))
        .with_width(14.)
        .with_height(14.)
        .finish();

        Container::new(
            ConstrainedBox::new(
                Flex::row()
                    .with_main_axis_size(MainAxisSize::Max)
                    .with_main_axis_alignment(MainAxisAlignment::SpaceBetween)
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(
                        Flex::row()
                            .with_cross_axis_alignment(CrossAxisAlignment::Center)
                            .with_child(icon_el)
                            .with_child(Container::new(label_el).with_margin_left(8.).finish())
                            .finish(),
                    )
                    .with_child(chevron)
                    .finish(),
            )
            .with_min_height(32.)
            .finish(),
        )
        .with_horizontal_padding(12.)
        .with_corner_radius(CornerRadius::with_all(Radius::Pixels(6.)))
        .with_border(Border::all(1.).with_border_color(internal_colors::neutral_4(theme)))
        .with_background(internal_colors::fg_overlay_1(theme))
        .finish()
    }

    fn render_agent_handler_options(
        &self,
        appearance: &Appearance,
        selected_agent: ThirdPartyAgentHandler,
    ) -> Box<dyn Element> {
        let mut column = Flex::column().with_cross_axis_alignment(CrossAxisAlignment::Stretch);
        for (index, agent) in ThirdPartyAgentHandler::ALL.iter().copied().enumerate() {
            let mouse_state = self
                .agent_command_option_mouse_states
                .get(index)
                .cloned()
                .unwrap_or_default();
            let row = self.render_agent_handler_option(
                appearance,
                agent,
                agent == selected_agent,
                mouse_state,
            );
            column = column.with_child(
                Container::new(row)
                    .with_margin_top(if index == 0 { 0. } else { 4. })
                    .finish(),
            );
        }
        column.finish()
    }

    fn render_agent_handler_option(
        &self,
        appearance: &Appearance,
        agent: ThirdPartyAgentHandler,
        is_selected: bool,
        mouse_state: MouseStateHandle,
    ) -> Box<dyn Element> {
        let theme = appearance.theme();
        let background_for_text = theme.background().into_solid();
        let text_color = if is_selected {
            internal_colors::accent_fg_strong(theme).into_solid()
        } else {
            internal_colors::text_sub(theme, background_for_text)
        };
        let background = if is_selected {
            Some(internal_colors::accent_overlay_3(theme))
        } else {
            None
        };
        let ui_font_family = appearance.ui_font_family();
        let label = agent.display_name().to_string();
        let icon = agent.icon();

        Hoverable::new(mouse_state, move |_| {
            let icon_el = ConstrainedBox::new(Box::new(icon.to_warpui_icon(text_color.into())))
                .with_width(14.)
                .with_height(14.)
                .finish();
            let label_el = Text::new(label.clone(), ui_font_family, 14.)
                .with_color(text_color)
                .with_style(Properties {
                    weight: Weight::Normal,
                    ..Default::default()
                })
                .with_line_height_ratio(1.0)
                .finish();

            let mut container = Container::new(
                Flex::row()
                    .with_cross_axis_alignment(CrossAxisAlignment::Center)
                    .with_child(icon_el)
                    .with_child(Container::new(label_el).with_margin_left(8.).finish())
                    .finish(),
            )
            .with_horizontal_padding(12.)
            .with_vertical_padding(8.)
            .with_corner_radius(CornerRadius::with_all(Radius::Pixels(6.)));
            if let Some(background) = background {
                container = container.with_background(background);
            }
            container.finish()
        })
        .with_cursor(Cursor::PointingHand)
        .on_click(move |ctx, _, _| {
            ctx.dispatch_typed_action(ThirdPartySlideAction::SelectAgentCommandHandler { agent });
        })
        .finish()
    }

    fn render_toolbar_section(
        &self,
        appearance: &Appearance,
        cli_toolbar_enabled: bool,
    ) -> Box<dyn Element> {
        let is_selected = self.selected_setting == Some(SettingCard::CliToolbar);

        let card = render_toggle_card(
            appearance,
            ToggleCardSpec {
                title: "CLI agent toolbar",
                is_expanded: is_selected,
                is_left_selected: cli_toolbar_enabled,
                left_label: "Enabled",
                right_label: "Disabled",
                card_mouse_state: self.cli_toolbar_card_mouse_state.clone(),
                on_expand: Box::new(|ctx, _, _| {
                    ctx.dispatch_typed_action(ThirdPartySlideAction::SelectSettingCard {
                        card: SettingCard::CliToolbar,
                    });
                }),
                left_mouse: self.cli_toolbar_seg_left_mouse.clone(),
                right_mouse: self.cli_toolbar_seg_right_mouse.clone(),
                on_left: Box::new(|ctx, _, _| {
                    ctx.dispatch_typed_action(ThirdPartySlideAction::SetCliAgentToolbarEnabled {
                        enabled: true,
                    });
                }),
                on_right: Box::new(|ctx, _, _| {
                    ctx.dispatch_typed_action(ThirdPartySlideAction::SetCliAgentToolbarEnabled {
                        enabled: false,
                    });
                }),
                chips: vec![],
            },
        );

        Container::new(
            Flex::column()
                .with_main_axis_size(MainAxisSize::Min)
                .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
                .with_child(card)
                .finish(),
        )
        .with_margin_top(16.)
        .finish()
    }

    fn render_notifications_section(
        &self,
        appearance: &Appearance,
        show_agent_notifications: bool,
    ) -> Box<dyn Element> {
        let is_selected = self.selected_setting == Some(SettingCard::Notifications);

        let card = render_toggle_card(
            appearance,
            ToggleCardSpec {
                title: "Notifications",
                is_expanded: is_selected,
                is_left_selected: show_agent_notifications,
                left_label: "Enabled",
                right_label: "Disabled",
                card_mouse_state: self.notifications_card_mouse_state.clone(),
                on_expand: Box::new(|ctx, _, _| {
                    ctx.dispatch_typed_action(ThirdPartySlideAction::SelectSettingCard {
                        card: SettingCard::Notifications,
                    });
                }),
                left_mouse: self.notifications_seg_left_mouse.clone(),
                right_mouse: self.notifications_seg_right_mouse.clone(),
                on_left: Box::new(|ctx, _, _| {
                    ctx.dispatch_typed_action(ThirdPartySlideAction::SetShowAgentNotifications {
                        enabled: true,
                    });
                }),
                on_right: Box::new(|ctx, _, _| {
                    ctx.dispatch_typed_action(ThirdPartySlideAction::SetShowAgentNotifications {
                        enabled: false,
                    });
                }),
                chips: vec![],
            },
        );

        Container::new(
            Flex::column()
                .with_main_axis_size(MainAxisSize::Min)
                .with_cross_axis_alignment(CrossAxisAlignment::Stretch)
                .with_child(card)
                .finish(),
        )
        .with_margin_top(16.)
        .finish()
    }

    fn render_bottom_nav(
        &self,
        appearance: &Appearance,
        intention: OnboardingIntention,
    ) -> Box<dyn Element> {
        let back_button = self.back_button.render(
            appearance,
            button::Params {
                content: button::Content::Label("Back".into()),
                theme: &button::themes::Naked,
                options: button::Options {
                    on_click: Some(Box::new(|ctx, _app, _pos| {
                        ctx.dispatch_typed_action(ThirdPartySlideAction::BackClicked);
                    })),
                    ..button::Options::default(appearance)
                },
            },
        );

        let enter = Keystroke::parse("enter").unwrap_or_default();
        let next_button = self.next_button.render(
            appearance,
            button::Params {
                content: button::Content::Label("Next".into()),
                theme: &button::themes::Primary,
                options: button::Options {
                    keystroke: Some(enter),
                    on_click: Some(Box::new(|ctx, _app, _pos| {
                        ctx.dispatch_typed_action(ThirdPartySlideAction::NextClicked);
                    })),
                    ..button::Options::default(appearance)
                },
            },
        );

        let is_terminal = matches!(intention, OnboardingIntention::Terminal);
        let (step_index, step_count) = if is_terminal { (2, 4) } else { (3, 5) };
        bottom_nav::onboarding_bottom_nav(
            appearance,
            step_index,
            step_count,
            Some(back_button),
            Some(next_button),
        )
    }

    fn render_visual(
        &self,
        cli_toolbar_enabled: bool,
        show_agent_notifications: bool,
        vertical: bool,
    ) -> Box<dyn Element> {
        if self.selected_setting == Some(SettingCard::Notifications) {
            let path = if show_agent_notifications {
                Self::VISUAL_IMAGE_PATHS[4]
            } else {
                Self::VISUAL_IMAGE_PATHS[5]
            };
            layout::onboarding_right_panel_with_bg(path, layout::FOREGROUND_LAYOUT_CODE_REVIEW)
        } else {
            let path = match (cli_toolbar_enabled, vertical) {
                (true, true) => Self::VISUAL_IMAGE_PATHS[0],
                (true, false) => Self::VISUAL_IMAGE_PATHS[1],
                (false, true) => Self::VISUAL_IMAGE_PATHS[2],
                (false, false) => Self::VISUAL_IMAGE_PATHS[3],
            };
            layout::onboarding_right_panel_with_bg(path, layout::FOREGROUND_LAYOUT_THIRD_PARTY)
        }
    }
}

impl Entity for ThirdPartySlide {
    type Event = ();
}

impl View for ThirdPartySlide {
    fn ui_name() -> &'static str {
        "ThirdPartySlide"
    }

    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        let agent_command_handler = self.agent_command_handler(app);
        let cli_toolbar_enabled = self.cli_agent_toolbar_enabled(app);
        let show_agent_notifications = self.show_agent_notifications(app);
        let intention = self.model_intention(app);
        let vertical = self
            .onboarding_state
            .as_ref(app)
            .ui_customization()
            .use_vertical_tabs;

        layout::static_left(
            || {
                self.render_content(
                    appearance,
                    agent_command_handler,
                    cli_toolbar_enabled,
                    show_agent_notifications,
                    intention,
                )
            },
            || self.render_visual(cli_toolbar_enabled, show_agent_notifications, vertical),
        )
    }
}

impl ThirdPartySlide {
    fn select_setting_card(&mut self, card: SettingCard, ctx: &mut ViewContext<Self>) {
        self.selected_setting = Some(card);
        ctx.notify();
    }

    fn next(&mut self, ctx: &mut ViewContext<Self>) {
        self.onboarding_state.update(ctx, |model, ctx| {
            model.next(ctx);
        });
    }
}

impl OnboardingSlide for ThirdPartySlide {
    fn on_up(&mut self, ctx: &mut ViewContext<Self>) {
        let new_card = match self.selected_setting {
            None => SettingCard::AgentCommand,
            Some(SettingCard::AgentCommand) => SettingCard::AgentCommand,
            Some(SettingCard::CliToolbar) => SettingCard::AgentCommand,
            Some(SettingCard::Notifications) => SettingCard::CliToolbar,
        };
        self.selected_setting = Some(new_card);
        ctx.notify();
    }

    fn on_down(&mut self, ctx: &mut ViewContext<Self>) {
        let is_terminal = matches!(self.model_intention(ctx), OnboardingIntention::Terminal);
        let new_card = match self.selected_setting {
            None => SettingCard::AgentCommand,
            Some(SettingCard::AgentCommand) => SettingCard::CliToolbar,
            Some(SettingCard::CliToolbar) => {
                if is_terminal {
                    SettingCard::Notifications
                } else {
                    SettingCard::CliToolbar
                }
            }
            Some(SettingCard::Notifications) => SettingCard::Notifications,
        };
        self.selected_setting = Some(new_card);
        ctx.notify();
    }

    fn on_left(&mut self, ctx: &mut ViewContext<Self>) {
        match self.selected_setting {
            Some(SettingCard::AgentCommand) => {
                let current = self.agent_command_handler(ctx);
                self.onboarding_state.update(ctx, |model, ctx| {
                    model.set_agent_command_handler(current.previous(), ctx);
                });
                ctx.notify();
            }
            Some(SettingCard::CliToolbar) => {
                self.onboarding_state.update(ctx, |model, ctx| {
                    model.set_cli_agent_toolbar_enabled(true, ctx);
                });
                ctx.notify();
            }
            Some(SettingCard::Notifications) => {
                self.onboarding_state.update(ctx, |model, ctx| {
                    model.set_show_agent_notifications(true, ctx);
                });
                ctx.notify();
            }
            None => {}
        }
    }

    fn on_right(&mut self, ctx: &mut ViewContext<Self>) {
        match self.selected_setting {
            Some(SettingCard::AgentCommand) => {
                let current = self.agent_command_handler(ctx);
                self.onboarding_state.update(ctx, |model, ctx| {
                    model.set_agent_command_handler(current.next(), ctx);
                });
                ctx.notify();
            }
            Some(SettingCard::CliToolbar) => {
                self.onboarding_state.update(ctx, |model, ctx| {
                    model.set_cli_agent_toolbar_enabled(false, ctx);
                });
                ctx.notify();
            }
            Some(SettingCard::Notifications) => {
                self.onboarding_state.update(ctx, |model, ctx| {
                    model.set_show_agent_notifications(false, ctx);
                });
                ctx.notify();
            }
            None => {}
        }
    }

    fn on_enter(&mut self, ctx: &mut ViewContext<Self>) {
        self.next(ctx);
    }
}

impl TypedActionView for ThirdPartySlide {
    type Action = ThirdPartySlideAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            ThirdPartySlideAction::SelectSettingCard { card } => {
                self.select_setting_card(*card, ctx);
            }
            ThirdPartySlideAction::SelectAgentCommandHandler { agent } => {
                let value = *agent;
                self.onboarding_state.update(ctx, |model, ctx| {
                    model.set_agent_command_handler(value, ctx);
                });
                ctx.notify();
            }
            ThirdPartySlideAction::SetCliAgentToolbarEnabled { enabled } => {
                let value = *enabled;
                self.onboarding_state.update(ctx, |model, ctx| {
                    model.set_cli_agent_toolbar_enabled(value, ctx);
                });
                ctx.notify();
            }
            ThirdPartySlideAction::SetShowAgentNotifications { enabled } => {
                let value = *enabled;
                self.onboarding_state.update(ctx, |model, ctx| {
                    model.set_show_agent_notifications(value, ctx);
                });
                ctx.notify();
            }
            ThirdPartySlideAction::BackClicked => {
                let onboarding_state = self.onboarding_state.clone();
                onboarding_state.update(ctx, |model, ctx| {
                    model.back(ctx);
                });
            }
            ThirdPartySlideAction::NextClicked => {
                self.next(ctx);
            }
        }
    }
}
