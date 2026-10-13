//! Read a locally imported profile once; absent or incompatible data keeps the fork's defaults.

use std::sync::OnceLock;

use super::data::{self, AbilityDef, ClassDef, Kind};
use survivors_reference::Profile;

struct Reference {
    profile: Profile,
    auto_shot: Option<AbilityDef>,
}

pub(super) fn initialize() {
    let _ = profile();
}

fn profile() -> Option<&'static Reference> {
    static PROFILE: OnceLock<Option<Reference>> = OnceLock::new();
    PROFILE
        .get_or_init(|| {
            let text = crate::webenv::var("WOW_SURVIVORS_REFERENCE").or_else(|| {
                let path = crate::local_state::home()?.join("survivors-reference.toml");
                crate::local_state::read_to_string(&path).ok()
            })?;
            match Profile::parse(&text) {
                Ok(profile)
                    if profile.classes().iter().all(|c| {
                        data::CLASSES.iter().any(|def| def.id == c.id)
                            && (data::ABILITIES.iter().any(|a| a.key == c.start)
                                || (c.id == 3 && c.start == "auto_shot" && c.projectile.is_some()))
                    }) =>
                {
                    bevy::log::info!(
                        source = profile.source_sha256(),
                        "survivors: loaded local reference class profile"
                    );
                    let auto_shot = profile.class(3).and_then(|c| {
                        let p = c.projectile.as_ref()?;
                        (c.start == "auto_shot").then(|| AbilityDef {
                            key: "auto_shot",
                            spell: p.spell,
                            kind: Kind::Bolt,
                            damage: p.damage,
                            cooldown: p.cooldown,
                            range: p.range,
                            count: p.count,
                            ..*data::ability("arcane_shot")
                        })
                    });
                    Some(Reference { profile, auto_shot })
                }
                result => {
                    bevy::log::warn!(
                        ?result,
                        "survivors: invalid reference profile; using defaults"
                    );
                    None
                }
            }
        })
        .as_ref()
}

pub(super) fn health(class: &ClassDef, default: f32) -> f32 {
    profile()
        .and_then(|p| p.profile.class(class.id))
        .map_or(default, |c| c.health)
}

pub(super) fn starting_ability(class: &ClassDef) -> &'static AbilityDef {
    let Some(reference) = profile() else {
        return data::ability(class.start);
    };
    let key = reference
        .profile
        .class(class.id)
        .map_or(class.start, |c| c.start.as_str());
    if key == "auto_shot" {
        if let Some(ability) = reference.auto_shot.as_ref() {
            return ability;
        }
    }
    data::ability(key)
}
