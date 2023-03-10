use std::collections::HashMap;

use bevy::prelude::*;
use iyes_loopless::state::CurrentState;
use sabi::prelude::Lobby;
use crate::{
    ui_pieces::*, player::prelude::{PlayerEvent, Player, MouseState}
};



pub struct UiPlugin;
impl Plugin for UiPlugin{
    fn build(&self, app: &mut App) {
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
    // turn into leafwing inventory slot hashmap later
    let mut inv : HashMap<u8, String> = HashMap::from([
        (1u8, "autoattack".to_owned()),
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
                            add_tooltip_info(&value)
                        ));
                    }
                });
            });
        });
        // Tooltip UI
        commands.spawn((
            tooltip(),
            Tooltip(TooltipInfo::default()),
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


fn hover_tooltips(
    mut query_tt: Query<(Entity, &mut Tooltip, &mut Style,  &Children, &mut Visibility), Without<TooltipInfo>>,
    mut query_hover: Query<(&TooltipInfo, &Interaction)>,
    mut query_children: Query<(&TooltipChild, Option<&mut Text>, Option<&mut UiImage>)>,
    children_query: Query<&Children>,
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
                for ( info,  inter) in query_hover.iter_mut() {
                    match inter {
                        Interaction::Hovered | Interaction::Clicked => {
                            new_vis = true;
                            for &child in children.iter() {
                                if let Ok((marker, text, image)) = query_children.get_mut(child) {
                                    match marker {
                                        TooltipChild::Title => {
                                            if let Some(mut text) = text {
                                                text.sections[0].value = info.title.clone();
                                            }
                                        }
                                        TooltipChild::Description => {
                                            if let Some(mut text) = text {
                                                text.sections[0].value = info.description.clone();
                                            }
                                        }
                                        TooltipChild::Image => {
                                            if let Some(mut image) = image {
                                                image.0 = asset_server.load(&info.image).clone();
                                            }
                                        }
                                    }
                                }
                            }                            
                        }
                        Interaction::None => {}
                    }
                }
                // need to set it after cus otherwise it will disable if no gap between hoverables, aka same frame `Changed` attr
                vis.is_visible = new_vis;
            }        
        }
    }
}