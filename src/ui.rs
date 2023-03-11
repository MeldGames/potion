use std::collections::HashMap;

use bevy::prelude::*;
use bevy_rapier3d::parry::simba::scalar::SupersetOf;
use iyes_loopless::state::CurrentState;
use sabi::prelude::Lobby;
use crate::{
    ui_pieces::*, player::prelude::{PlayerEvent, Player, MouseState}
};

#[derive(Clone)]
pub struct ListStat{
    statname: String,
    amount: f32,
    icon: String,
}

#[derive(Clone)]
pub struct Item{
    name: String,
    description: Option<String>,
    stat1: ListStat,
    image: String,
}



pub struct UiPlugin;
impl Plugin for UiPlugin{
    fn build(&self, app: &mut App) {
        app.insert_resource(Events::<HoverEvent>::default());
        app
            .add_system(hover_tooltips)
            .add_system(add_player_ui);
    }
}

fn add_player_ui(
    mut commands:Commands,
    query: Query<Entity, Added<Player>>,
    asset_server: Res<AssetServer>,
    mut player_reader: EventReader<PlayerEvent>,
    mut lobby: ResMut<Lobby>,
){  
    let flamestat = ListStat{
        statname: "Flame Damage".to_owned(),
        amount: 20.0,
        icon: "icons/mage.png".to_owned(),
    };
    
    let cleave_data = Item{
        name:"Power Cleave".to_owned(),
        description:Some("Swing in a large arc, knocking back enemies".to_owned()),
        stat1: flamestat.clone(),
        image: "icons/autoattack.png".to_owned(),
    };
    let dash_data = Item{
        name:"Driving Strike".to_owned(),
        description:Some("cool desc".to_owned()),
        stat1: flamestat.clone(),
        image: "icons/dash.png".to_owned(),
    };
    
    let itemdata = HashMap::from([
        ("cleave", cleave_data),
        ("dash", dash_data)
    ]);
    // turn into leafwing inventory slot hashmap later
    let mut inv : HashMap<u8, String> = HashMap::from([
        (1u8, "cleave".to_owned()),
        (2u8, "dash".to_owned()),
    ]);

    for (_e) in query.iter() {
        // HUD
        commands.spawn((
            player_root(),
            Name::new("Player UI")
        ))
        .with_children(|parent| {
            parent.spawn( player_bottom_container())
            .with_children(|parent| {
                parent.spawn((
                    item_holder(),
                    ItemHolder,
                ))
                .with_children(|parent| {
                    for (_key, value) in &inv {                    
                        parent.spawn((
                            item_image(&asset_server, format!("icons/{}.png", value).to_owned()),
                            Interaction::None,
                            add_tooltip_info(&value),
                            Name::new(value.clone()),
                        ));
                    }
                });
            });
        });
        // Tooltip UI
        commands.spawn((
            tooltip(),
            Tooltip(TooltipInfo::default()),
            Hovering(false),
            Name::new("Tooltip"),
        )).with_children(|parent| {
            parent.spawn((
                tooltip_desc(&asset_server),
                TooltipChild::Description));
            parent.spawn((
                tooltip_title(&asset_server),
                TooltipChild::Title));
            parent.spawn((
                tooltip_image(
                    &asset_server,
                    "icons/autoattack.png".to_string(),
                ),
                TooltipChild::Image,));
        });
    }
}

pub struct HoverEvent(TooltipInfo);

fn change_tooltip(
    mut query_tt: Query<(Entity, &mut Tooltip, &mut Hovering, &mut Style,  &Children, &mut Visibility), Without<TooltipInfo>>,
    query_hover: Query<(&TooltipInfo, &Interaction), Changed<Interaction>>,
    mut hoversend: EventWriter<HoverEvent>,
){
    let mut changed = false;
    for ( info, inter) in query_hover.iter() {
        match inter {
            Interaction::Hovered | Interaction::Clicked => {
                if let Ok((e, mut tt, mut hover, mut style,  children, mut vis)) = query_tt.get_single_mut() {
                    hover.0 = changed;
                    tt.0 = info.clone();
                }
                hoversend.send(HoverEvent(info.clone()));
                changed = true;                        
            }
            Interaction::None => {}
        }
    }
}


fn hover_tooltips(
    mut query_tt: Query<(Entity, &mut Tooltip, &mut Style,  &Children, &mut Visibility), Without<TooltipInfo>>,
    query_hover: Query<(&TooltipInfo, &Interaction), Changed<Interaction>>,
    mut query_children: Query<(&TooltipChild, Option<&mut Text>, Option<&mut UiImage>)>,
    mut query_stats: Query<(&TooltipStats)>,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
    state: Res<CurrentState<MouseState>>,
){
    if let Ok((e, mut tt, mut style,  children, mut vis)) = query_tt.get_single_mut() {
        if state.0 == MouseState::Locked{
            vis.is_visible = false;
            return;
        }
        if let Some(window) = windows.get_primary() {
            if let Some(cursor_pos) = window.cursor_position(){
                style.position.left = Val::Px(cursor_pos.x);
                style.position.bottom = Val::Px(cursor_pos.y);
                let mut new_vis = false; 
                let mut new_info = add_tooltip_info("autoattack");

                for ( info,  inter) in query_hover.iter() {
                    match inter {
                        Interaction::Hovered | Interaction::Clicked => {
                            new_vis = true;
                            new_info = info.clone();
                            println!("updating");                         
                        }
                        Interaction::None => {}
                    }
                    vis.is_visible = new_vis;
                }
                // need to set it after cus otherwise it will disable if no gap between hoverables, aka same frame `Changed` attr
                if tt.0.title != new_info.title && new_vis{
                    tt.0 = new_info.clone();
                    for &child in children.iter() {
                        if let Ok((marker, text, image)) = query_children.get_mut(child) {
                            match marker {
                                TooltipChild::Title => {
                                    if let Some(mut text) = text {
                                        text.sections[0].value = new_info.title.clone();
                                    }
                                }
                                TooltipChild::Description => {
                                    if let Some(mut text) = text {
                                        text.sections[0].value = new_info.description.clone();
                                    }
                                }
                                TooltipChild::Image => {
                                    if let Some(mut image) = image {
                                        image.0 = asset_server.load(&new_info.image).clone();
                                    }
                                }
                            }
                        }
                    }  
                } 
            }        
        }
    }
}