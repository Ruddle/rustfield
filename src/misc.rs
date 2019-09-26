use ggez::nalgebra as na;

pub type Vector2 = na::Vector2<f32>;

use std::{thread, time};
pub fn sleep() {
    thread::sleep(time::Duration::from_millis(1000));
}
