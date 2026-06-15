use crate::types::Player;

#[derive(Debug, Default, Clone)]
pub struct InputState {
    pub forward:    bool,
    pub backward:   bool,
    pub yaw_left:   bool,
    pub yaw_right:  bool,
    pub pitch_up:   bool,
    pub pitch_down: bool,
    pub roll_left:  bool,
    pub roll_right: bool,
    pub fire:       bool,
}

impl InputState {
    pub fn apply_to_player(&self, player: &mut Player) {
        player.move_forward    = if self.forward    { 1.0 } else { 0.0 };
        player.move_backward   = if self.backward   { 1.0 } else { 0.0 };
        player.move_left       = if self.yaw_left   { 1.0 } else { 0.0 };
        player.move_right      = if self.yaw_right  { 1.0 } else { 0.0 };
        player.move_up         = if self.pitch_up   { 1.0 } else { 0.0 };
        player.move_down       = if self.pitch_down { 1.0 } else { 0.0 };
        player.move_pitchleft  = if self.roll_left  { 1.0 } else { 0.0 };
        player.move_pitchright = if self.roll_right { 1.0 } else { 0.0 };
    }

    pub fn clear(&mut self) {
        *self = Self::default();
    }
}
