//
// button.rs
// Copyright (C) 2022 matthew <matthew@matthew-ubuntu>
// Distributed under terms of the MIT license.
//

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext};

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
pub enum MouseAction {
    Drag,
    Extrude,
}

#[derive(Debug)]
pub struct MouseOptions {
    pub action: MouseAction,
    pub lock_z: bool,
}

#[derive(Debug, Component, Clone, Copy, PartialEq, Eq)]
pub enum BoolOption {
    LockZ,
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const CLEAR: Color = Color::rgba(0., 0., 0., 0.);

pub fn button_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.insert_resource(MouseOptions {
        action: MouseAction::Drag,
        lock_z: true,
    });
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                size: Size::new(Val::Auto, Val::Percent(100.)),
                display: Display::Flex,
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::ColumnReverse,
                justify_content: JustifyContent::FlexStart,
                padding: Rect::all(Val::Px(10.)),
                ..Default::default()
            },
            color: UiColor(CLEAR.into()),
            ..Default::default()
        })
        .with_children(|commands| {
            let text_font = TextStyle {
                font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                font_size: 40.0,
                color: Color::rgb(0.9, 0.9, 0.9),
            };

            commands
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: PRESSED_BUTTON.into(),
                    ..Default::default()
                })
                .insert(MouseAction::Drag)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section("Drag", text_font.clone(), Default::default()),
                        style: Style {
                            margin: Rect {
                                left: Val::Px(10.),
                                right: Val::Px(10.),
                                top: Val::Px(5.),
                                bottom: Val::Px(5.),
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                });

            commands
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: NORMAL_BUTTON.into(),
                    ..Default::default()
                })
                .insert(MouseAction::Extrude)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section("Extrude", text_font.clone(), Default::default()),
                        style: Style {
                            margin: Rect {
                                left: Val::Px(10.),
                                right: Val::Px(10.),
                                top: Val::Px(5.),
                                bottom: Val::Px(5.),
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                });

            commands
                .spawn_bundle(ButtonBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..Default::default()
                    },
                    color: PRESSED_BUTTON.into(),
                    ..Default::default()
                })
                .insert(BoolOption::LockZ)
                .with_children(|parent| {
                    parent.spawn_bundle(TextBundle {
                        text: Text::with_section("Lock Z", text_font.clone(), Default::default()),
                        style: Style {
                            margin: Rect {
                                left: Val::Px(10.),
                                right: Val::Px(10.),
                                top: Val::Px(5.),
                                bottom: Val::Px(5.),
                            },
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                });
        });
}

pub fn button_system(
    mut interaction_query: Query<(&Interaction, &mut UiColor, &MouseAction), (Without<BoolOption>, With<Button>)>,
    mut booleans: Query<(&Interaction, &mut UiColor, &BoolOption), (Without<MouseAction>, With<Button>)>,
    mut opts: ResMut<MouseOptions>,
) {
    for (interaction, mut color, action) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                opts.action = *action;
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                if *action == opts.action {
                    *color = PRESSED_BUTTON.into();
                } else {
                    *color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {
                if *action == opts.action {
                    *color = PRESSED_BUTTON.into();
                } else {
                    *color = NORMAL_BUTTON.into();
                }
            }
        }
    }

    for (interaction, mut color, action) in booleans.iter_mut() {
        let opt = match action {
            BoolOption::LockZ => &mut opts.lock_z,
        };
        match interaction {
            Interaction::Clicked => {
                *opt = !*opt;
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                if *opt {
                    *color = PRESSED_BUTTON.into();
                } else {
                    *color = HOVERED_BUTTON.into();
                }
            }
            Interaction::None => {
                if *opt {
                    *color = PRESSED_BUTTON.into();
                } else {
                    *color = NORMAL_BUTTON.into();
                }
            }
        }
    }
}
