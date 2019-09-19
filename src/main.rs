//! The simplest possible example that does something.

use ggez;
use ggez::conf::{Conf, NumSamples, WindowSetup};
use ggez::{event, GameResult};
use rustfield::MainState;

pub fn main() -> GameResult {
    let hidpi_factor: f32;
    {
        // Create a dummy window so we can get monitor scaling information
        let cb = ggez::ContextBuilder::new("", "");
        let (_ctx, events_loop) = &mut cb.build()?;
        hidpi_factor = events_loop.get_primary_monitor().get_hidpi_factor() as f32;
    }

    let cb = ggez::ContextBuilder::new("rustfield", "ruddle").conf(Conf {
        window_setup: WindowSetup {
            title: String::from("rustfield"),
            samples: NumSamples::Zero,
            vsync: false,
            icon: String::new(),
            srgb: true,
        },
        ..ggez::conf::Conf::new()
    });

    let (ctx, event_loop) = &mut cb.build()?;
    let mut state = MainState::new(ctx, hidpi_factor)?;
    event::run(ctx, event_loop, &mut state)
}
