
use bevy::prelude::*;

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const CLEAR: Color = Color::rgba(0., 0., 0., 0.);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(button_setup);
    }
}

fn button_setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
}

#[derive(Debug, Clone, Bundle)]
pub struct MenuBundle<ID: Component> {
    id: ID,
    #[bundle]
    node: NodeBundle,
}

impl<ID: Component> MenuBundle<ID> {
    pub fn new(id: ID) -> Self {
        Self {
            id,
            node: NodeBundle {
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
            },
        }
    }
}



#[derive(Component)]
pub struct Bool<T: Send + Sync + 'static>(bool, T);

impl<T: Send + Sync + 'static> Bool<T> {
    pub fn id(&self) -> &T {
        &self.1
    }

    pub fn val(&self) -> bool {
        self.0
    }
}

pub fn selected<T>(bt: &Query<&Bool<T>>, item: &T) -> bool
    where T: PartialEq + Send + Sync + 'static 
{
    bt.iter().find(|b| b.id() == item).map(|b| b.val()).unwrap_or(false)
}

pub fn option(
    commands: &mut ChildBuilder<'_, '_, '_>,
    font: &Handle<Font>,
    name: &str,
    action: impl Send + Sync + 'static,
    default: bool
) {
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
        .insert(Bool(default, action))
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(name, TextStyle {
                    font: font.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                }, Default::default()),
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
}

#[derive(Component)]
pub struct Radio<T: TypeID + Send + Sync + 'static>(bool, T);

pub fn radio(
    commands: &mut ChildBuilder<'_, '_, '_>,
    font: &Handle<Font>,
    name: &str,
    action: impl TypeID + Send + Sync + 'static,
    default: bool,
) {
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
        .insert(Radio(default, action))
        .with_children(|parent| {
            parent.spawn_bundle(TextBundle {
                text: Text::with_section(name, TextStyle {
                    font: font.clone(),
                    font_size: 40.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                }, Default::default()),
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
}

// #[derive(Debug, Clone, Copy, PatrialEq, Eq, Component)]
// pub struct Radio<ID, T>(ID, T);

// pub fn button(commands: &mut ChildBuilder<'_, '_, '_>, name: &str, action: impl Component) {
//     commands
//         .spawn_bundle(ButtonBundle {
//             style: Style {
//                 justify_content: JustifyContent::Center,
//                 align_items: AlignItems::Center,
//                 ..Default::default()
//             },
//             color: PRESSED_BUTTON.into(),
//             ..Default::default()
//         })
//         .insert(action)
//         .with_children(|parent| {
//             parent.spawn_bundle(TextBundle {
//                 text: Text::with_section(name, TextStyle {
//                     font,
//                     font_size: 40.0,
//                     color: Color::rgb(0.9, 0.9, 0.9),
//                 }, Default::default()),
//                 style: Style {
//                     margin: Rect {
//                         left: Val::Px(10.),
//                         right: Val::Px(10.),
//                         top: Val::Px(5.),
//                         bottom: Val::Px(5.),
//                     },
//                     ..Default::default()
//                 },
//                 ..Default::default()
//             });
//         });
// }