use bevy::prelude::*;

//
// Player UI Components
//

pub fn player_root() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(350.), Val::Auto),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            border: UiRect{
                left: Val::Px(2.0),
                top: Val::Px(2.0),
                ..default()
            },
            ..default()
        },
        background_color: Color::rgba(0.5, 0.7, 0.7, 0.0).into(),
        ..default()
    }
}


pub fn player_bottom_container() -> NodeBundle {
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Auto),
            margin: UiRect {
                bottom: Val::Px(0.),
                ..default()
            },
            flex_direction: FlexDirection::Column,
            align_self: AlignSelf::FlexEnd,
            ..default()
        },
        background_color: Color::rgba(0.5, 0.7, 0.7, 0.0).into(),
        ..default()
    }
}

#[derive(Component, Debug)]
pub struct ItemHolder;

pub fn item_holder() -> NodeBundle {
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            margin: UiRect {
                bottom: Val::Px(10.),
                ..default()
            },
            ..default()
        },
        background_color: Color::rgba(1., 1., 1., 0.0).into(),
        ..default()
    }
}

pub fn item_image(asset_server: &Res<AssetServer>, path: String) -> ImageBundle {
    ImageBundle {
        style: Style {
            size: Size::new(Val::Px(64.), Val::Px(64.)),
            ..default()
        },
        image: asset_server.load(&path).into(),
        ..default()
    }
}

pub fn item_icon(asset_server: &Res<AssetServer>, path: String) -> ImageBundle {
    ImageBundle {
        style: Style {
            size: Size::new(Val::Px(16.), Val::Px(16.)),
            ..default()
        },
        image: asset_server.load(&path).into(),
        ..default()
    }
}


// Tooltip 

pub fn tooltip() -> NodeBundle {
    NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Baseline,
            margin: UiRect {
                bottom: Val::Px(10.),
                ..default()
            },
            padding: UiRect::all(Val::Px(20.)),
            ..default()
        },
        background_color: Color::rgba(0., 0., 0., 1.0).into(),
        transform: Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        visibility: Visibility{
            is_visible: false
        },
        z_index: ZIndex::Global(2),
        ..default()
    }
}

pub fn tooltip_title(asset_server: &Res<AssetServer>) -> TextBundle {
    TextBundle {
        style: Style {
            position: UiRect {
                ..default()
            },
            ..default()
        },
        text: Text::from_section(
            "title",
            TextStyle {
                font: asset_server.load("fonts/Proxima/proximanova-semibold.otf"),
                font_size: 32.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    }
}

pub fn tooltip_desc(asset_server: &Res<AssetServer>) -> TextBundle {
    TextBundle {
        style: Style {
            margin: UiRect {
                top: Val::Px(15.),
                left: Val::Px(0.),
                ..default()
            },
            position: UiRect {
                ..default()
            },
            max_size: Size::new(Val::Px(320.), Val::Auto),
            ..default()
        },
        text: Text::from_section(
            "sabington stop biting my finger PLEASE",
            TextStyle {
                font: asset_server.load("fonts/Proxima/proximanova-regular.otf"),
                font_size: 22.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    }
}

#[derive(Component, Debug)]
pub enum TooltipChild {
    Title,
    Description,
    Image,
}

#[derive(Component)]
pub struct TooltipStats;

#[derive(Component, Debug, Default, Clone)]
pub struct TooltipInfo {
    pub title: String,
    pub image: String,
    pub description: String,
}

#[derive(Component, Debug)]
pub struct Hovering(pub bool);

#[derive(Component, Debug)]
pub struct Tooltip(pub TooltipInfo);

pub fn tooltip_image(asset_server: &Res<AssetServer>, path: String) -> ImageBundle {
    ImageBundle {
        style: Style {
            size: Size::new(Val::Px(64.), Val::Px(64.)),
            position_type: PositionType::Absolute,
            position: UiRect {
                right: Val::Px(24.),
                top: Val::Px(-25.),
                ..default()
            },
            ..default()
        },
        image: asset_server.load(&path).into(),
        ..default()
    }
}

pub fn add_tooltip_info(item: &str) -> TooltipInfo {
    match item {
        "autoattack" => TooltipInfo {
            title: "Cleave".to_string(),
            image: "icons/autoattack.png".to_string(),
            description: "bam as fuck".to_string(),
        },
        "dash" => TooltipInfo {
            title: "Driving Strike".to_string(),
            image: "icons/dash.png".to_string(),
            description: "Hercules delivers a mighty strike, driving all enemies back, damaging and Stunning them. Hercules is immune to Knockback during the dash.".to_string(),
        },
        _ => TooltipInfo {
            title: "Ability".to_string(),
            image: "icons/autoattack.png".to_string(),
            description: "A very boring attack".to_string(),
        },
    }
}