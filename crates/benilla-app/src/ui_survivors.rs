//! The local run's live state, presented through the stock FrameXML player HUD.

use benilla_protocol::messages::{ActionButton, ObjectFields, ObjectType, ACTION_KIND_SPELL};
use benilla_ui::script::{ActionState, UiScript};
use bevy::prelude::*;

use super::hero::Hero;
use super::{data, Phase, Run};
use crate::net::{FieldChanged, Guid, GuidIndex, ObjectStore, SelfGuid};

#[derive(Resource, Default)]
pub(super) struct StockHudReady(pub(super) bool);

pub(super) fn feed_player(
    run: Res<Run>,
    mut heroes: Query<(Entity, &Guid, &mut ObjectStore, &mut crate::net::UnitSpeeds), With<Hero>>,
    mut me: ResMut<SelfGuid>,
    mut index: ResMut<GuidIndex>,
    mut names: ResMut<crate::names::NameCache>,
    mut actions: ResMut<crate::ui_action::PlayerActions>,
    mut edges: MessageWriter<FieldChanged>,
    mut named: Local<Option<u64>>,
) {
    let Ok((entity, guid, mut store, mut speeds)) = heroes.single_mut() else {
        if let Some(old) = named.take() {
            index.0.remove(&old);
        }
        if me.0.is_some() {
            me.0 = None;
        }
        return;
    };
    if me.0 != Some(guid.0) {
        me.0 = Some(guid.0);
    }
    index.0.insert(guid.0, entity);
    if *named != Some(guid.0) {
        if let Some(old) = *named {
            index.0.remove(&old);
        }
        *named = Some(guid.0);
        names.insert_player(guid.0, "Hero".into(), None);
    }
    let stats = run.stats();
    let speed = stats.speed * run.haste_mult();
    let next_speeds = benilla_protocol::MoveSpeeds {
        walk: speed * (2.5 / 7.0),
        run: speed,
        run_back: speed * (4.5 / 7.0),
        swim: speed * (4.722222 / 7.0),
        swim_back: speed * (2.5 / 7.0),
        turn_rate: std::f32::consts::PI,
    };
    if speeds.0 != next_speeds {
        speeds.0 = next_speeds;
    }
    let pairs = [
        (2, 0x19), // OBJECT | UNIT | PLAYER; the body still uses its chosen display model.
        (22, run.hp.max(0.0).ceil() as u32),
        (28, stats.max_hp.ceil() as u32),
        (34, run.level),
        (194, 2 << 24), // Normal (unrested) XP; this local mode has no rest areas.
        (716, run.xp.max(0.0) as u32),
        (717, data::xp_to_next(run.level) as u32),
        (1175, 0),        // No rested XP pool.
        (1261, u32::MAX), // No watched reputation faction.
    ];
    let mut changed = Vec::new();
    store.bypass_change_detection().0.merge_diff(
        ObjectFields::from_pairs(&pairs),
        |i, old, new| {
            changed.push(FieldChanged {
                entity,
                guid: guid.0,
                kind: ObjectType::Player,
                index: i,
                old,
                new,
            });
        },
    );
    if !changed.is_empty() {
        store.set_changed();
        edges.write_batch(changed);
    }
    let spells = run.abilities.iter().map(|o| o.def.spell).collect();
    if actions.spells != spells {
        actions
            .buttons
            .retain(|_, button| run.abilities.iter().any(|o| o.def.spell == button.action));
        for ability in &run.abilities {
            if !actions
                .buttons
                .values()
                .any(|b| b.action == ability.def.spell)
            {
                if let Some(slot) = (0u8..12).find(|s| !actions.buttons.contains_key(s)) {
                    actions.buttons.insert(
                        slot,
                        ActionButton {
                            slot,
                            action: ability.def.spell,
                            kind: ACTION_KIND_SPELL,
                        },
                    );
                }
            }
        }
        actions.spells = spells;
        actions.dirty = true;
    }
}

pub(super) fn feed_hud(
    script: Option<NonSendMut<UiScript>>,
    run: Res<Run>,
    phase: Res<State<Phase>>,
    clock: Res<crate::ui_script::UiClock>,
    actions: Res<crate::ui_action::PlayerActions>,
    mut previous: Local<Vec<(u32, f32)>>,
    mut options: ResMut<super::view::ViewOptions>,
    mut next: ResMut<NextState<Phase>>,
    mut ready: ResMut<StockHudReady>,
    pending: Option<Res<crate::ui_script::PendingEntryUiLoad>>,
) {
    ready.0 = pending.is_none()
        && script
            .as_ref()
            .is_some_and(|s| s.widget_kind("MainMenuBar").is_some());
    if !ready.0 {
        return;
    }
    let Some(mut script) = script else { return };
    let initialized = script.widget_kind("SurvivorsViewButton").is_none();
    if initialized {
        crate::ui_script::run_or_warn(&script, VIEW_BUTTON);
        // Offline runs have no realm/character chat-cache restore to raise this stock init event.
        script.fire_event("UPDATE_CHAT_WINDOWS", vec![]);
    }
    if script
        .eval::<bool>("local v = SurvivorsChangeView; SurvivorsChangeView = nil; return v == true")
        .unwrap_or(false)
    {
        options.toggle();
    }
    if script
        .eval::<bool>(
            "local v = SurvivorsCharacterSelect; SurvivorsCharacterSelect = nil; return v == true",
        )
        .unwrap_or(false)
    {
        crate::ui_script::run_or_warn(&script, "if GameMenuFrame then GameMenuFrame:Hide() end");
        next.set(Phase::Menu);
    } else if *phase.get() == Phase::Playing && script.frame_visible("GameMenuFrame") {
        next.set(Phase::Paused);
    } else if *phase.get() == Phase::Paused && !script.frame_visible("GameMenuFrame") {
        next.set(Phase::Playing);
    }
    let label = match options.mode {
        super::view_rules::ViewMode::ThirdPerson => {
            "SurvivorsViewButton:SetText('View: third person (F6)')"
        }
        super::view_rules::ViewMode::TopDown => {
            "SurvivorsViewButton:SetText('View: top-down (F6)')"
        }
    };
    if initialized || options.is_changed() {
        crate::ui_script::run_or_warn(&script, label);
    }
    let visible = !matches!(phase.get(), Phase::Menu | Phase::GameOver);
    if script.frame_visible("UIParent") != visible {
        let command = if visible {
            "if UIParent then UIParent:Show() end"
        } else {
            "if UIParent then UIParent:Hide() end"
        };
        if let Err(error) = script.run(command) {
            warn!(%error, "survivors: stock HUD visibility");
        }
    }
    if !visible {
        return;
    }
    let stats = run.stats();
    let mut changed =
        initialized || previous.len() != run.abilities.len() || *phase.get() != Phase::Playing;
    for (&slot, button) in &actions.buttons {
        let Some(owned) = run.abilities.iter().find(|o| o.def.spell == button.action) else {
            continue;
        };
        let total = owned.cooldown(&stats).max(owned.cd);
        let cooldown = (owned.cd > 0.0).then(|| {
            (
                ((clock.ui_now - f64::from(total - owned.cd)) * 1000.0).max(1.0) as i64,
                (total * 1000.0) as u32,
                true,
            )
        });
        script.set_action_state(
            u32::from(slot) + 1,
            Some(ActionState {
                usable: run.dead_t.is_none(),
                has_range: owned.def.range > 0.0,
                cooldown,
                ..Default::default()
            }),
        );
        changed |= previous
            .iter()
            .find(|(spell, _)| *spell == owned.def.spell)
            .is_none_or(|(_, cd)| owned.cd > *cd + 0.03 || (*cd > 0.0 && owned.cd <= 0.0));
    }
    if changed {
        script.fire_event("ACTIONBAR_UPDATE_COOLDOWN", vec![]);
    }
    *previous = run.abilities.iter().map(|o| (o.def.spell, o.cd)).collect();
}

// Own adapter code, using the stock widgets and templates loaded from the player's MPQs.
const VIEW_BUTTON: &str = r#"
if GameMenuFrame then
    local b = CreateFrame("Button", "SurvivorsViewButton", GameMenuFrame, "UIPanelButtonTemplate")
    b:SetWidth(210)
    b:SetHeight(24)
    b:SetPoint("TOPLEFT", GameMenuFrame, "TOPRIGHT", 8, -10)
    b:SetScript("OnClick", function() SurvivorsChangeView = true end)
    if GameMenuButtonLogout then
        GameMenuButtonLogout:SetText("Character selection")
        GameMenuButtonLogout:SetScript("OnClick", function() SurvivorsCharacterSelect = true end)
    end
end
"#;

#[derive(Component)]
pub(super) struct SurvivalStatus;

/// Keep the survival timer and announcements above the stock HUD, without duplicate unit bars.
pub(super) fn detach_status(
    mut commands: Commands,
    timers: Query<Entity, With<super::hud::TimerText>>,
    kills: Query<Entity, With<super::hud::KillText>>,
    banners: Query<Entity, With<super::hud::BannerText>>,
) {
    for (entities, top) in [
        (timers.iter().collect::<Vec<_>>(), 6.0),
        (kills.iter().collect(), 44.0),
        (banners.iter().collect(), 70.0),
    ] {
        for entity in entities {
            commands.entity(entity).remove::<ChildOf>().insert((
                SurvivalStatus,
                GlobalZIndex(400),
                TextLayout::new_with_justify(Justify::Center),
                bevy::ui::FocusPolicy::Pass,
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(top),
                    left: Val::ZERO,
                    width: Val::Percent(100.0),
                    ..default()
                },
            ));
        }
    }
}

pub(super) fn show_status(
    phase: Res<State<Phase>>,
    mut status: Query<(&mut Visibility, &mut Node), With<SurvivalStatus>>,
) {
    let visible = !matches!(phase.get(), Phase::Menu | Phase::GameOver);
    for (mut v, mut node) in &mut status {
        *v = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        node.display = if visible {
            Display::Flex
        } else {
            Display::None
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_damage_levels_spells_and_haste_reach_the_stock_player_feeds() {
        let mut app = App::new();
        app.init_resource::<Run>()
            .init_resource::<SelfGuid>()
            .init_resource::<GuidIndex>()
            .init_resource::<crate::names::NameCache>()
            .init_resource::<crate::ui_action::PlayerActions>()
            .add_message::<FieldChanged>()
            .add_systems(Update, feed_player);
        let hero = app
            .world_mut()
            .spawn((
                Hero { vy: 0.0, air: 0.0 },
                Guid(42),
                ObjectStore(ObjectFields::from_pairs(&[])),
                crate::net::UnitSpeeds(benilla_protocol::MoveSpeeds::default()),
            ))
            .id();
        {
            let mut run = app.world_mut().resource_mut::<Run>();
            run.hp = 73.4;
            run.level = 5;
            run.xp = 17.0;
            run.mobile.haste = 1.6;
            run.mobile.haste_t = 3.0;
            run.abilities.push(super::super::Owned {
                def: data::ability("fireball"),
                level: 1,
                cd: 2.0,
            });
        }
        app.update();
        assert_eq!(app.world().resource::<SelfGuid>().0, Some(42));
        let fields = &app.world().get::<ObjectStore>(hero).unwrap().0;
        assert_eq!(fields.unit_health(), Some(74));
        assert_eq!(fields.unit_level(), Some(5));
        assert_eq!(fields.player_xp(), Some(17));
        let run = app.world().resource::<Run>();
        assert_eq!(
            fields.unit_max_health(),
            Some(run.stats().max_hp.ceil() as u32)
        );
        assert_eq!(
            fields.player_next_level_xp(),
            Some(data::xp_to_next(5) as u32)
        );
        assert!(
            (app.world()
                .get::<crate::net::UnitSpeeds>(hero)
                .unwrap()
                .0
                .run
                - run.stats().speed * 1.6)
                .abs()
                < 0.001
        );
        let spell = data::ability("fireball").spell;
        assert!(app
            .world()
            .resource::<crate::ui_action::PlayerActions>()
            .spells
            .contains(&spell));
        // A dragged action keeps its slot while the run learns an additional spell.
        {
            let mut actions = app
                .world_mut()
                .resource_mut::<crate::ui_action::PlayerActions>();
            let mut button = actions.buttons.remove(&0).unwrap();
            button.slot = 7;
            actions.buttons.insert(7, button);
        }
        app.world_mut()
            .resource_mut::<Run>()
            .abilities
            .push(super::super::Owned {
                def: data::ability("cleave"),
                level: 1,
                cd: 0.0,
            });
        app.update();
        assert_eq!(
            app.world()
                .resource::<crate::ui_action::PlayerActions>()
                .buttons[&7]
                .action,
            spell
        );
        app.world_mut().despawn(hero);
        app.update();
        assert_eq!(app.world().resource::<SelfGuid>().0, None);
        assert!(!app.world().resource::<GuidIndex>().0.contains_key(&42));
    }
}
