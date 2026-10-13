//! Optional reference data supplied locally; this crate embeds no original game content.

use serde::Deserialize;
use std::collections::HashSet;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    schema: u32,
    source_sha256: String,
    classes: Vec<Class>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Class {
    pub id: u8,
    pub health: f32,
    pub start: String,
    pub projectile: Option<Projectile>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Projectile {
    pub spell: u32,
    pub damage: f32,
    pub cooldown: f32,
    pub range: f32,
    pub count: u32,
}

impl Profile {
    /// Reject a damaged or incompatible profile as a whole; never partially apply it.
    pub fn parse(text: &str) -> Result<Self, String> {
        let profile: Self = toml::from_str(text).map_err(|e| e.to_string())?;
        if profile.schema != 1 {
            return Err("unsupported reference profile schema".into());
        }
        if profile.source_sha256.len() != 64
            || !profile.source_sha256.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("reference profile must identify its source SHA-256".into());
        }
        if profile.classes.is_empty() || profile.classes.len() > 9 {
            return Err("reference profile must contain one to nine classes".into());
        }
        let mut seen = HashSet::new();
        for class in &profile.classes {
            if ![1, 2, 3, 4, 5, 7, 8, 9, 11].contains(&class.id) || !seen.insert(class.id) {
                return Err("reference profile has an unknown or duplicate class".into());
            }
            if !class.health.is_finite() || !(1.0..=100_000.0).contains(&class.health) {
                return Err("reference profile has invalid starting health".into());
            }
            if class.start.is_empty()
                || class.start.len() > 64
                || !class
                    .start
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b == b'_')
            {
                return Err("reference profile has an invalid starting ability key".into());
            }
            if let Some(p) = &class.projectile {
                if p.spell == 0
                    || !p.damage.is_finite()
                    || !(0.0..=1_000_000.0).contains(&p.damage)
                    || !p.cooldown.is_finite()
                    || !(0.05..=3600.0).contains(&p.cooldown)
                    || !p.range.is_finite()
                    || !(0.0..=1000.0).contains(&p.range)
                    || !(1..=64).contains(&p.count)
                {
                    return Err("reference profile has an invalid starting projectile".into());
                }
            }
        }
        Ok(profile)
    }

    pub fn class(&self, id: u8) -> Option<&Class> {
        self.classes.iter().find(|c| c.id == id)
    }

    pub fn classes(&self) -> &[Class] {
        &self.classes
    }

    pub fn source_sha256(&self) -> &str {
        &self.source_sha256
    }
}

#[cfg(test)]
mod tests {
    use super::Profile;

    fn profile(rows: &str) -> String {
        format!("schema = 1\nsource_sha256 = {:?}\n{rows}", "a".repeat(64))
    }

    #[test]
    fn looks_up_by_class_id_rather_than_position() {
        let p = Profile::parse(&profile(
            "[[classes]]\nid = 8\nhealth = 999.0\nstart = \"fireball\"\n\
             [[classes]]\nid = 1\nhealth = 777.0\nstart = \"cleave\"\n",
        ))
        .unwrap();
        assert_eq!(p.class(1).unwrap().health, 777.0);
        assert_eq!(p.class(8).unwrap().start, "fireball");
        assert!(p.class(11).is_none());
    }

    #[test]
    fn rejects_invalid_health_and_duplicate_classes() {
        for health in ["nan", "inf", "0.0", "-1.0", "100001.0"] {
            let row = format!("[[classes]]\nid = 8\nhealth = {health}\nstart = \"fireball\"\n");
            assert!(Profile::parse(&profile(&row)).is_err(), "{health}");
        }
        let row = "[[classes]]\nid = 8\nhealth = 999.0\nstart = \"fireball\"\n";
        assert!(Profile::parse(&profile(&format!("{row}{row}"))).is_err());
    }

    #[test]
    fn rejects_incompatible_or_unidentified_data() {
        let valid = profile("[[classes]]\nid = 8\nhealth = 999.0\nstart = \"fireball\"\n");
        assert!(Profile::parse(&valid.replace("schema = 1", "schema = 2")).is_err());
        assert!(Profile::parse(&valid.replace(&"a".repeat(64), "unknown")).is_err());
        assert!(Profile::parse(&valid.replace("id = 8", "id = 6")).is_err());
        assert!(Profile::parse(&valid.replace("fireball", "../file")).is_err());
        assert!(Profile::parse(&format!("{valid}extra = true\n")).is_err());
    }

    #[test]
    fn validates_an_optional_starting_projectile() {
        let text = profile(
            "[[classes]]\nid = 3\nhealth = 999.0\nstart = \"auto_shot\"\n\
             [classes.projectile]\nspell = 75\ndamage = 2.0\ncooldown = 3.0\nrange = 4.0\ncount = 1\n",
        );
        let p = Profile::parse(&text).unwrap();
        assert_eq!(p.class(3).unwrap().projectile.as_ref().unwrap().spell, 75);
        for invalid in [
            text.replace("count = 1", "count = 0"),
            text.replace("damage = 2.0", "damage = nan"),
            text.replace("cooldown = 3.0", "cooldown = 0.0"),
        ] {
            assert!(Profile::parse(&invalid).is_err());
        }
    }
}
