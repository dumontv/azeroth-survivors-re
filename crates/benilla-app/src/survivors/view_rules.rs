//! The saved view choice and the handoff between the stock controller and the top-down rig.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum ViewMode {
    #[default]
    ThirdPerson,
    TopDown,
}

impl ViewMode {
    pub(super) fn parse(text: &str) -> Option<Self> {
        match text.trim() {
            "third-person" => Some(Self::ThirdPerson),
            "top-down" => Some(Self::TopDown),
            _ => None,
        }
    }

    pub(super) fn saved(self) -> &'static str {
        match self {
            Self::ThirdPerson => "third-person",
            Self::TopDown => "top-down",
        }
    }

    pub(super) fn toggled(self) -> Self {
        match self {
            Self::ThirdPerson => Self::TopDown,
            Self::TopDown => Self::ThirdPerson,
        }
    }

    pub(super) fn stock_controls(self, playing: bool) -> bool {
        self == Self::ThirdPerson && playing
    }
}

#[cfg(test)]
mod tests {
    use super::ViewMode;

    #[test]
    fn fresh_players_use_third_person_and_saved_choices_round_trip() {
        assert_eq!(ViewMode::default(), ViewMode::ThirdPerson);
        for mode in [ViewMode::ThirdPerson, ViewMode::TopDown] {
            assert_eq!(ViewMode::parse(mode.saved()), Some(mode));
            assert_eq!(mode.toggled().toggled(), mode);
        }
        assert_eq!(ViewMode::parse("old-or-damaged-value"), None);
    }

    #[test]
    fn only_a_playing_third_person_run_owns_the_stock_controller() {
        assert!(ViewMode::ThirdPerson.stock_controls(true));
        assert!(!ViewMode::ThirdPerson.stock_controls(false));
        assert!(!ViewMode::TopDown.stock_controls(true));
        assert!(!ViewMode::TopDown.stock_controls(false));
    }
}
