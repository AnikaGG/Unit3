pub struct Animation {
    // states are sprite sheet positions
    pub states: Vec<[f32; 6]>,
    // frame counter is how many frames have passed on the current animation state
    pub frame_counter: i32,
    // rate is how many frames need to pass to go to the next animation state
    pub rate: i32,
    // state_number is which frame of the animation we're on
    pub state_number: usize,

    pub is_facing_left: bool,
    pub sprite_width: f32,

    pub is_looping: bool,
    pub is_done: bool,
}

impl Animation {
    pub fn tick(&mut self){
        // iterate frame counter
        self.frame_counter += 1;

        // if enough frames have passed, go to the next frame of the animation
        if self.frame_counter > self.rate {
            self.state_number += 1;

            if self.is_looping {
                // if we've gone past the last frame of the animation, go back to the first frame
                if self.state_number >= self.states.len() as usize - 1 {
                    self.state_number = 0;
                }
            }
            else {
                if self.state_number >= self.states.len() as usize - 1 {
                    self.is_done = true;
                }
                else {
                    self.is_done = false;
                }
            }

            self.frame_counter = 0;
        }


    }
    pub fn stop(&mut self){
        while self.state_number != 0 {
            self.tick();
        }
    }
    pub fn get_current_state(&mut self) -> [f32; 6]{

        if !self.is_looping && self.state_number > self.states.len() - 1 {
            self.state_number = self.states.len() - 1;

        }
        if self.is_facing_left{
            if self.states[self.state_number][2] > 0.0 {
                self.states[self.state_number][2] *= -1.0;
                self.states[self.state_number][0] += self.sprite_width;
            }
              
        }
        else {
            if self.states[self.state_number][2] < 0.0 {
                self.states[self.state_number][2] *= -1.0;
                self.states[self.state_number][0] -= self.sprite_width;
            }            
        }

        return self.states[self.state_number]
    }

    pub fn apply_face_left(&mut self){
        self.is_facing_left = true;
    }

    pub fn apply_face_right(&mut self){
        self.is_facing_left = false;
    }

    pub fn restart_animation(&mut self){
        self.frame_counter = 0;
        self.state_number = 0;
    }
}