use ggez::conf::{FullscreenType, WindowMode};
use ggez::event::{EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self};
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
mod flowfield;
mod imgui_wrapper;
mod misc;
mod ui_impl;
use crate::flowfield::{CellPos, FlowField};
use crate::misc::Vector2;
use ggez::graphics::Rect;
use imgui_wrapper::ImGuiWrapper;
use ui_impl::UI;
mod sprite;
use sprite::AllSprite;

const GRID_CELL_SIZE: f32 = 8.0;

pub struct MainState {
    imgui_wrapper: ImGuiWrapper,
    hidpi_factor: f32,
    flowfield: FlowField,
    ui: UI,
    sprite: AllSprite,
}

impl MainState {
    pub fn new(mut ctx: &mut Context, hidpi_factor: f32) -> GameResult<MainState> {
        let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
        let mut s = MainState {
            hidpi_factor,
            imgui_wrapper,
            flowfield: FlowField::new(CellPos::new()),
            ui: UI::new(),
            sprite: AllSprite::new(ctx)?,
        };

        use std::time::{Duration, Instant};
        let now = Instant::now();
        {
            let mut conti = true;
            while (conti) {
                conti = !s.flowfield.step();
            }
        }

        println!("{}", now.elapsed().as_millis());

        graphics::set_mode(
            ctx,
            WindowMode {
                width: 1600.0,
                height: 900.0,
                maximized: false,
                fullscreen_type: FullscreenType::Windowed,
                borderless: false,
                min_width: 0.0,
                max_width: 0.0,
                min_height: 0.0,
                max_height: 0.0,
                resizable: true,
            },
        );

        graphics::set_screen_coordinates(
            ctx,
            Rect {
                x: 0.0,
                y: 0.0,
                w: 1600.0,
                h: 900.0,
            },
        );

        Ok(s)
    }

    fn compute_all(&mut self) {
        let mut conti = true;
        while (conti) {
            conti = !self.flowfield.step();
        }
    }
}

impl MainState {
    fn half_screen(&mut self, ctx: &mut Context) -> Vector2 {
        let [w, h] = self.imgui_wrapper.imgui.io().display_size;
        Vector2::new(w / 2.0, h / 2.0)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        if self.ui.compute_all {
            self.compute_all();
        }

        if self.ui.compute_live {
            self.flowfield.step();
        }

        if self.ui.compute_step {
            self.ui.compute_step = false;
            self.flowfield.step();
        }

        self.ui.zoom_smooth = self.ui.zoom * 0.1 + self.ui.zoom_smooth * 0.9;
        self.ui.cam_pos_smooth = self.ui.cam_pos * 0.1 + self.ui.cam_pos_smooth * 0.9;

        let half_screen = self.half_screen(_ctx);
        self.ui.mouse_pos_camera =
            -self.ui.cam_pos_smooth + (self.ui.mouse_pos - half_screen) / self.ui.zoom_smooth;

        let shift_mult = if self.ui.keys_pressed.contains(&KeyCode::LShift) {
            6.0
        } else {
            2.0
        };

        for key_pressed in &self.ui.keys_pressed {
            match key_pressed {
                KeyCode::Z => self.ui.cam_pos += Vector2::new(0.0, 1.0) * shift_mult,
                KeyCode::S => self.ui.cam_pos += Vector2::new(0.0, -1.0) * shift_mult,
                KeyCode::Q => self.ui.cam_pos += Vector2::new(1.0, 0.0) * shift_mult,
                KeyCode::D => self.ui.cam_pos += Vector2::new(-1.0, 0.0) * shift_mult,
                _ => {}
            }
        }

        // Cost drawing
        let cell_pos = CellPos {
            i: ((self.ui.mouse_pos_camera.x / GRID_CELL_SIZE) as usize)
                .min(flowfield::GRID_SIZE - 1),
            j: ((self.ui.mouse_pos_camera.y / GRID_CELL_SIZE) as usize)
                .min(flowfield::GRID_SIZE - 1),
        };

        let big_cell_pos = flowfield::Field::<i32>::grow(&cell_pos);

        if !self.imgui_wrapper.imgui.io().want_capture_mouse {
            match self.ui.flowfield_mode {
                ui_impl::DisplayFlowField::Cost => {
                    if self.ui.mouse_pressed.contains(&MouseButton::Left) {
                        for cell_pos in &big_cell_pos {
                            self.flowfield.cost.set(&cell_pos, 200);
                        }
                    }
                    if self.ui.mouse_pressed.contains(&MouseButton::Right) {
                        for cell_pos in &big_cell_pos {
                            self.flowfield.cost.set(&cell_pos, 1);
                        }
                    }
                    if self.ui.mouse_pressed.contains(&MouseButton::Middle) {
                        self.flowfield.reset();
                    }
                }
                ui_impl::DisplayFlowField::Integration => {
                    if self.ui.mouse_pressed.contains(&MouseButton::Left) {
                        self.flowfield.set_objective(cell_pos);
                        if self.ui.compute_all {
                            self.compute_all();
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 0.01].into());

        //Camera param

        let point = na::Point2::from(self.ui.cam_pos_smooth);

        let half_screen = self.half_screen(ctx);

        let param = graphics::DrawParam::new()
            .dest(point * self.ui.zoom_smooth + half_screen)
            .offset(na::Point2::new(0.0, 0.0))
            .scale(na::Vector2::new(self.ui.zoom_smooth, self.ui.zoom_smooth));

        //Drawing FLOWFIELD
        fn cell_pos_2_rect(cell_pos: &CellPos) -> Rect {
            let i = cell_pos.i;
            let j = cell_pos.j;
            Rect {
                x: i as f32 * GRID_CELL_SIZE,
                y: j as f32 * GRID_CELL_SIZE,
                w: GRID_CELL_SIZE,
                h: GRID_CELL_SIZE,
            }
        }

        let mut color_vec: Vec<u8> = Vec::new();
        for j in 0..flowfield::GRID_SIZE {
            for i in 0..flowfield::GRID_SIZE {
                let (i, j) = (i as f32, j as f32);

                fn color_of(i: i32, accel: f32) -> f32 {
                    (1.0 - f32::exp(-i as f32 * accel)) * 0.8
                        + (0.5 + f32::cos((accel * 10.0 + 1.0) * i as f32 / 10.0) * 0.5).powi(4)
                            * 0.2
                }

                let v = match self.ui.flowfield_mode {
                    ui_impl::DisplayFlowField::Cost => self.flowfield.cost.get(&(i, j).into()),
                    ui_impl::DisplayFlowField::Integration => {
                        self.flowfield.integration.get(&(i, j).into())
                    }
                };

                color_vec.push((color_of(v, 0.01) * 255.0) as u8);
                color_vec.push((color_of(v, 0.007) * 255.0) as u8);
                color_vec.push((color_of(v, 0.004) * 255.0) as u8);
                color_vec.push(255);
            }
        }

        let mut img = ggez::graphics::Image::from_rgba8(
            ctx,
            flowfield::GRID_SIZE as u16,
            flowfield::GRID_SIZE as u16,
            &color_vec[..],
        )?;

        img.set_filter(ggez::graphics::FilterMode::Nearest);
        graphics::draw(
            ctx,
            &img,
            graphics::DrawParam::new()
                .dest(point * self.ui.zoom_smooth + half_screen)
                //            .rotation(20.0 / 100.0)
                .offset(na::Point2::new(0.0, 0.0))
                .scale(na::Vector2::new(
                    self.ui.zoom_smooth * GRID_CELL_SIZE as f32,
                    self.ui.zoom_smooth * GRID_CELL_SIZE as f32,
                )),
        )?;

        for visit in &self.flowfield.to_visit {
            let color = [0.0, 0.2, 1.0, 0.5].into();

            let rectangle = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                cell_pos_2_rect(visit),
                color,
            )?;
            graphics::draw(ctx, &rectangle, param)?;
        }

        //Flow arrow
        if self.ui.flowfield_show_arrow {
            for j in 0..flowfield::GRID_SIZE {
                for i in 0..flowfield::GRID_SIZE {
                    let (i, j) = (i as f32, j as f32);

                    let v = &self.flowfield.flow.get(&(i, j).into());
                    let x = (v % 3) - 1;
                    let y = (v / 3) - 1;

                    let p = graphics::DrawParam::new().dest(na::Point2::new(
                        i * GRID_CELL_SIZE + (GRID_CELL_SIZE - 3.0) / 2.0,
                        j * GRID_CELL_SIZE + (GRID_CELL_SIZE - 3.0) / 2.0,
                    ));

                    if x * y > 0 {
                        self.sprite.diag_se.add(p);
                    } else if x * y < 0 {
                        self.sprite.diag_ne.add(p);
                    } else if x != 0 {
                        self.sprite.hori.add(p);
                    } else if y != 0 {
                        self.sprite.vert.add(p);
                    };
                }
            }

            graphics::draw(ctx, &self.sprite.hori, param)?;
            graphics::draw(ctx, &self.sprite.vert, param)?;
            graphics::draw(ctx, &self.sprite.diag_ne, param)?;
            graphics::draw(ctx, &self.sprite.diag_se, param)?;
            self.sprite.clear();
        }
        //CASE POINTED
        let color = [0.0, 1.0, 0.2, 0.5].into();
        let cell_pos = CellPos {
            i: (self.ui.mouse_pos_camera.x / GRID_CELL_SIZE) as usize,
            j: (self.ui.mouse_pos_camera.y / GRID_CELL_SIZE) as usize,
        };
        let rectangle = graphics::Mesh::new_rectangle(
            ctx,
            graphics::DrawMode::fill(),
            cell_pos_2_rect(&cell_pos),
            color,
        )?;
        graphics::draw(ctx, &rectangle, param)?;

        //POINTER
        let circle = graphics::Mesh::new_circle(
            ctx,
            graphics::DrawMode::fill(),
            na::Point2::from(self.ui.mouse_pos_camera),
            2.0,
            0.05,
            graphics::BLACK,
        )?;
        graphics::draw(ctx, &circle, param)?;

        // Render game ui
        self.imgui_wrapper
            .render(ctx, self.hidpi_factor, &mut self.ui);

        graphics::present(ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(
        &mut self,
        _ctx: &mut Context,
        button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((
            button == MouseButton::Left,
            button == MouseButton::Right,
            button == MouseButton::Middle,
        ));

        self.ui.mouse_pressed.insert(button);
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((false, false, false));
        self.ui.mouse_pressed.remove(&_button);
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);

        self.ui.mouse_pos = Vector2::new(x, y);
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {
        self.ui.zoom = f32::max(0.1, self.ui.zoom * (1.0 + _y / 10.0))
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymods: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            //            KeyCode::P => {}
            _ => (),
        }

        self.ui.keys_pressed.insert(keycode);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymods: KeyMods) {
        self.ui.keys_pressed.remove(&_keycode);
    }

    fn resize_event(&mut self, _ctx: &mut Context, _width: f32, _height: f32) {
        graphics::set_screen_coordinates(
            _ctx,
            Rect {
                x: 0.0,
                y: 0.0,
                w: _width,
                h: _height,
            },
        );
    }
}
