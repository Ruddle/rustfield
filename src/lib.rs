use ggez::conf::{FullscreenType, WindowMode};
use ggez::event::{EventHandler, KeyCode, KeyMods, MouseButton};
use ggez::graphics::{self, Rect};
use ggez::nalgebra as na;
use ggez::{Context, GameResult};
mod field;
mod flowfield;
mod imgui_wrapper;
mod map;
mod misc;
mod pathfinding;
mod sprite;
mod ui_impl;
use crate::field::{CellPos, Field};
use crate::flowfield::{FlowField, FlowFieldState, GRID_SIZE};
use crate::map::Map;
use crate::misc::Vector2;
use crate::pathfinding::{AStarCompute, PathComputer};
use imgui_wrapper::ImGuiWrapper;
use sprite::AllSprite;
use std::collections::HashSet;
use ui_impl::HighLevelUI;

const MAP_SIZE: usize = 300;
const GRID_CELL_SIZE: f32 = 8.0;

pub struct MainState {
    imgui_wrapper: ImGuiWrapper,
    hidpi_factor: f32,
    map: Map,
    sprite: AllSprite,
    path_computer: PathComputer,
}

impl EventHandler for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        let bools: Vec<bool> = self.ui().astars.iter().map(|x| x.0).collect();

        for (index, to_delete) in bools.iter().enumerate() {
            if *to_delete {
                self.path_computer.astars.remove(index);
            }
        }

        fn formatAstar(astar: &AStarCompute) -> String {
            match astar {
                AStarCompute::Computing {
                    from,
                    to,
                    sort_cost,
                    ..
                } => format!(
                    "({},{})->({},{}) cost: {}",
                    astar.from_to().0.i,
                    astar.from_to().0.j,
                    astar.from_to().1.i,
                    astar.from_to().1.j,
                    sort_cost
                ),
                _ => format!(
                    "({},{})->({},{})",
                    astar.from_to().0.i,
                    astar.from_to().0.j,
                    astar.from_to().1.i,
                    astar.from_to().1.j,
                ),
            }
        }

        self.ui_mut().astars = self
            .path_computer
            .astars
            .iter()
            .map(|astar| (false, formatAstar(astar)))
            .collect();

        if self.ui_mut().compute_all {
            self.compute_all();
        }

        if self.ui_mut().compute_live {
            for _ in 0..50 {
                self.compute_live();
            }
        }

        if self.ui_mut().compute_step {
            self.ui_mut().compute_step = false;
            self.compute_step();
        }

        self.ui_mut().zoom_smooth = self.ui_mut().zoom * 0.1 + self.ui_mut().zoom_smooth * 0.9;
        self.ui_mut().cam_pos_smooth =
            self.ui_mut().cam_pos * 0.1 + self.ui_mut().cam_pos_smooth * 0.9;

        let half_screen = self.half_screen(_ctx);
        self.ui_mut().mouse_pos_camera = -self.ui_mut().cam_pos_smooth
            + (self.ui_mut().mouse_pos - half_screen) / self.ui_mut().zoom_smooth;

        let zoom_move_mult = 1.0 / self.ui().zoom_smooth;

        let shift_mult = if self.ui_mut().keys_pressed.contains(&KeyCode::LShift) {
            6.0
        } else {
            2.0
        };

        for key_pressed in self.ui_mut().keys_pressed.clone() {
            match key_pressed {
                KeyCode::Z => {
                    self.ui_mut().cam_pos += Vector2::new(0.0, 1.0) * shift_mult * zoom_move_mult
                }
                KeyCode::S => {
                    self.ui_mut().cam_pos += Vector2::new(0.0, -1.0) * shift_mult * zoom_move_mult
                }
                KeyCode::Q => {
                    self.ui_mut().cam_pos += Vector2::new(1.0, 0.0) * shift_mult * zoom_move_mult
                }
                KeyCode::D => {
                    self.ui_mut().cam_pos += Vector2::new(-1.0, 0.0) * shift_mult * zoom_move_mult
                }
                _ => {}
            }
        }

        // Cost drawing
        let cell_pos = CellPos {
            i: ((self.ui_mut().mouse_pos_camera.x / GRID_CELL_SIZE) as usize)
                .min(self.map.size - 1),
            j: ((self.ui_mut().mouse_pos_camera.y / GRID_CELL_SIZE) as usize)
                .min(self.map.size - 1),
        };

        let big_cell_pos = self.map.cost.grow(&cell_pos);

        let mouse_triggered_or_pressed = self.ui().mouse_pressed_or_triggered();

        let mouse_triggered: HashSet<MouseButton> =
            self.ui().mouse_trigger.iter().copied().collect();

        if !self.imgui_wrapper.imgui.io().want_capture_mouse {
            match self.ui().cursor_control {
                ui_impl::CursorControl::CostDrawing => {
                    if mouse_triggered_or_pressed.contains(&MouseButton::Left) {
                        for cell_pos in &big_cell_pos {
                            self.map.cost.set(&cell_pos, 200);
                        }
                    }
                    if mouse_triggered_or_pressed.contains(&MouseButton::Right) {
                        for cell_pos in &big_cell_pos {
                            self.map.cost.set(&cell_pos, 1);
                        }
                    }
                    if mouse_triggered.contains(&MouseButton::Middle) {
                        self.map.reset();
                    }
                }
                ui_impl::CursorControl::TripSetting => {
                    if mouse_triggered.contains(&MouseButton::Left) {
                        self.path_computer.astars.push(PathComputer::astar(
                            CellPos::new(),
                            cell_pos,
                            self.map.cost.clone(),
                        ));
                    }
                    if mouse_triggered.contains(&MouseButton::Right) {
                        //                        self.flowfield.set_objective(cell_pos);
                        if self.ui().compute_all {
                            self.compute_all();
                        }
                    }
                    if mouse_triggered.contains(&MouseButton::Middle) {
                        //                        self.flowfield.set_objective(cell_pos);
                        if self.ui().compute_all {
                            self.compute_all();
                        }
                    }
                }
            }
        }

        self.ui_mut().reset_trigger();

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, [0.1, 0.2, 0.3, 0.01].into());

        //Camera param

        self.draw_map(ctx);

        self.draw_path_computer(ctx);

        let point = na::Point2::from(self.ui_mut().cam_pos_smooth);

        let half_screen = self.half_screen(ctx);

        let param = graphics::DrawParam::new()
            .dest(point * self.ui_mut().zoom_smooth + half_screen)
            .offset(na::Point2::new(0.0, 0.0))
            .scale(na::Vector2::new(
                self.ui_mut().zoom_smooth,
                self.ui_mut().zoom_smooth,
            ));

        //CASE POINTED
        let color = [0.0, 1.0, 0.2, 0.5].into();
        let cell_pos = CellPos {
            i: (self.ui_mut().mouse_pos_camera.x / GRID_CELL_SIZE) as usize,
            j: (self.ui_mut().mouse_pos_camera.y / GRID_CELL_SIZE) as usize,
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
            na::Point2::from(self.ui_mut().mouse_pos_camera),
            2.0,
            0.05,
            graphics::BLACK,
        )?;
        graphics::draw(ctx, &circle, param)?;

        // Render game ui
        self.imgui_wrapper.render(ctx, self.hidpi_factor);

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

        self.ui_mut().mouse_pressed.insert(button);
        self.ui_mut().mouse_trigger.insert(button);
    }

    fn mouse_button_up_event(
        &mut self,
        _ctx: &mut Context,
        _button: MouseButton,
        _x: f32,
        _y: f32,
    ) {
        self.imgui_wrapper.update_mouse_down((false, false, false));
        self.ui_mut().mouse_pressed.remove(&_button);
    }

    fn mouse_motion_event(&mut self, _ctx: &mut Context, x: f32, y: f32, _dx: f32, _dy: f32) {
        self.imgui_wrapper.update_mouse_pos(x, y);

        self.ui_mut().mouse_pos = Vector2::new(x, y);
    }

    fn mouse_wheel_event(&mut self, _ctx: &mut Context, _x: f32, _y: f32) {
        self.ui_mut().zoom = f32::max(0.1, self.ui_mut().zoom * (1.0 + _y / 10.0))
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

        self.ui_mut().keys_pressed.insert(keycode);
    }

    fn key_up_event(&mut self, _ctx: &mut Context, _keycode: KeyCode, _keymods: KeyMods) {
        self.ui_mut().keys_pressed.remove(&_keycode);
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
        )
        .unwrap();
    }
}

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

impl MainState {
    pub fn new(mut ctx: &mut Context, hidpi_factor: f32) -> GameResult<MainState> {
        let imgui_wrapper = ImGuiWrapper::new(&mut ctx);
        let mut s = MainState {
            hidpi_factor,
            imgui_wrapper,
            map: Map::new(MAP_SIZE),
            sprite: AllSprite::new(ctx)?,
            path_computer: PathComputer::new(),
        };

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
        )?;

        for _ in 0..20 {
            s.path_computer.astars.push(PathComputer::astar(
                CellPos::new(),
                CellPos {
                    i: MAP_SIZE - 1,
                    j: MAP_SIZE / 2,
                },
                s.map.cost.clone(),
            ));
        }

        Ok(s)
    }

    fn ui_mut(&mut self) -> &mut HighLevelUI {
        &mut self.imgui_wrapper.ui
    }

    fn ui(&self) -> &HighLevelUI {
        &self.imgui_wrapper.ui
    }

    fn compute_all(&mut self) {
        let mut computed_anything = false;
        let start = std::time::Instant::now();
        for astar in &mut self.path_computer.astars {
            while match astar {
                AStarCompute::Computed { .. } => false,
                _ => true,
            } {
                computed_anything = true;
                AStarCompute::step_replace(astar);
            }
        }
        if computed_anything {
            self.ui_mut().last_compute_ms = start.elapsed().as_millis()
        }
    }

    fn compute_live(&mut self) {
        self.compute_step();
    }

    fn compute_step(&mut self) {
        for astar in &mut self.path_computer.astars {
            AStarCompute::step_replace(astar);
        }
        ()
    }

    fn half_screen(&mut self, _ctx: &mut Context) -> Vector2 {
        let [w, h] = self.imgui_wrapper.imgui.io().display_size;
        Vector2::new(w / 2.0, h / 2.0)
    }

    fn draw_path_computer(&mut self, ctx: &mut Context) -> GameResult<()> {
        let point = na::Point2::from(self.ui_mut().cam_pos_smooth);

        let half_screen = self.half_screen(ctx);

        let mut color_vec: Vec<u8> = vec![0; self.map.size * self.map.size * 4];

        fn color_pixel(cell_pos: &CellPos, color: &[f64], size: usize, color_vec: &mut Vec<u8>) {
            let init = (cell_pos.i + cell_pos.j * size) * 4;
            for (index, col) in color.iter().enumerate() {
                color_vec[init + index] = (col * 255.0) as u8
            }
        }

        for astar in &self.path_computer.astars {
            match astar {
                AStarCompute::Computing {
                    from,
                    to,
                    open_nodes,
                    closed_nodes,
                    ..
                } => {
                    color_pixel(from, &[0.0, 1.0, 0.0, 1.0], self.map.size, &mut color_vec);
                    color_pixel(to, &[1.0, 1.0, 0.0, 1.0], self.map.size, &mut color_vec);
                    for (_, node) in open_nodes {
                        color_pixel(
                            &node.cell_pos,
                            &[0.5, 0.5, 0.5, 0.2],
                            self.map.size,
                            &mut color_vec,
                        )
                    }
                    for node in closed_nodes {
                        color_pixel(&node, &[1.0, 0.0, 0.0, 0.2], self.map.size, &mut color_vec)
                    }
                    for min in open_nodes.iter().map(|x| x.1).min_by_key(|x| x.f()) {
                        color_pixel(
                            &min.cell_pos,
                            &[1.0, 0.0, 1.0, 1.0],
                            self.map.size,
                            &mut color_vec,
                        );
                    }
                }

                AStarCompute::Computed { from, to, path } => {
                    color_pixel(from, &[0.0, 1.0, 0.0, 1.0], self.map.size, &mut color_vec);
                    color_pixel(to, &[1.0, 1.0, 0.0, 1.0], self.map.size, &mut color_vec);
                    for node in path {
                        color_pixel(&node, &[0.0, 0.0, 1.0, 0.5], self.map.size, &mut color_vec);
                    }
                }
                _ => {}
            }
        }

        let mut img = ggez::graphics::Image::from_rgba8(
            ctx,
            self.map.size as u16,
            self.map.size as u16,
            &color_vec[..],
        )?;

        img.set_filter(ggez::graphics::FilterMode::Nearest);
        graphics::draw(
            ctx,
            &img,
            graphics::DrawParam::new()
                .dest(point * self.ui_mut().zoom_smooth + half_screen)
                //            .rotation(20.0 / 100.0)
                .offset(na::Point2::new(0.0, 0.0))
                .scale(na::Vector2::new(
                    self.ui_mut().zoom_smooth * GRID_CELL_SIZE as f32,
                    self.ui_mut().zoom_smooth * GRID_CELL_SIZE as f32,
                )),
        )?;

        Ok(())
    }

    fn draw_map(&mut self, ctx: &mut Context) -> GameResult<()> {
        let point = na::Point2::from(self.ui_mut().cam_pos_smooth);

        let half_screen = self.half_screen(ctx);

        let param = graphics::DrawParam::new()
            .dest(point * self.ui_mut().zoom_smooth + half_screen)
            .offset(na::Point2::new(0.0, 0.0))
            .scale(na::Vector2::new(
                self.ui_mut().zoom_smooth,
                self.ui_mut().zoom_smooth,
            ));

        let mut color_vec: Vec<u8> = Vec::new();

        let (min, max) = (self.map.cost.min() as i32, self.map.cost.max() as i32);

        for j in 0..self.map.size {
            for i in 0..self.map.size {
                let (i, j) = (i as f32, j as f32);

                fn color_of(i: f32, accel: f32) -> f32 {
                    (1.0 - f32::exp(-f32::powf(i, accel))) / 0.63
                }

                let v = self.map.cost.get(&(i, j).into()) as i32;
                let v = (v - min) as f32 / (max - min) as f32;

                color_vec.push((color_of(v, 0.5) * 255.0) as u8);
                color_vec.push((color_of(v, 1.1) * 255.0) as u8);
                color_vec.push((color_of(v, 2.0) * 255.0) as u8);
                color_vec.push(255);
            }
        }

        let mut img = ggez::graphics::Image::from_rgba8(
            ctx,
            self.map.size as u16,
            self.map.size as u16,
            &color_vec[..],
        )?;

        img.set_filter(ggez::graphics::FilterMode::Nearest);
        graphics::draw(
            ctx,
            &img,
            graphics::DrawParam::new()
                .dest(point * self.ui_mut().zoom_smooth + half_screen)
                //            .rotation(20.0 / 100.0)
                .offset(na::Point2::new(0.0, 0.0))
                .scale(na::Vector2::new(
                    self.ui_mut().zoom_smooth * GRID_CELL_SIZE as f32,
                    self.ui_mut().zoom_smooth * GRID_CELL_SIZE as f32,
                )),
        )?;

        Ok(())
    }

    fn draw_flowfield(&mut self, ctx: &mut Context, flowfield: &FlowField) -> GameResult<()> {
        let point = na::Point2::from(self.ui_mut().cam_pos_smooth);

        let half_screen = self.half_screen(ctx);

        let param = graphics::DrawParam::new()
            .dest(point * self.ui_mut().zoom_smooth + half_screen)
            .offset(na::Point2::new(0.0, 0.0))
            .scale(na::Vector2::new(
                self.ui_mut().zoom_smooth,
                self.ui_mut().zoom_smooth,
            ));

        let mut color_vec: Vec<u8> = Vec::new();

        let (min, max) = match self.ui_mut().flowfield_mode {
            ui_impl::DisplayFlowField::Cost => {
                (flowfield.cost.min() as i32, flowfield.cost.max() as i32)
            }
            ui_impl::DisplayFlowField::Integration => {
                (flowfield.integration.min(), flowfield.integration.max())
            }
        };

        for j in 0..GRID_SIZE {
            for i in 0..GRID_SIZE {
                let (i, j) = (i as f32, j as f32);

                fn color_of(i: f32, accel: f32) -> f32 {
                    (1.0 - f32::exp(-f32::powf(i, accel))) / 0.63
                }

                let v = match self.ui_mut().flowfield_mode {
                    ui_impl::DisplayFlowField::Cost => flowfield.cost.get(&(i, j).into()) as i32,
                    ui_impl::DisplayFlowField::Integration => {
                        flowfield.integration.get(&(i, j).into())
                    }
                };

                let v = (v - min) as f32 / (max - min) as f32;

                color_vec.push((color_of(v, 0.5) * 255.0) as u8);
                color_vec.push((color_of(v, 1.1) * 255.0) as u8);
                color_vec.push((color_of(v, 2.0) * 255.0) as u8);
                color_vec.push(255);
            }
        }

        let mut img = ggez::graphics::Image::from_rgba8(
            ctx,
            GRID_SIZE as u16,
            GRID_SIZE as u16,
            &color_vec[..],
        )?;

        img.set_filter(ggez::graphics::FilterMode::Nearest);
        graphics::draw(
            ctx,
            &img,
            graphics::DrawParam::new()
                .dest(point * self.ui_mut().zoom_smooth + half_screen)
                //            .rotation(20.0 / 100.0)
                .offset(na::Point2::new(0.0, 0.0))
                .scale(na::Vector2::new(
                    self.ui_mut().zoom_smooth * GRID_CELL_SIZE as f32,
                    self.ui_mut().zoom_smooth * GRID_CELL_SIZE as f32,
                )),
        )?;

        for visit in &flowfield.to_visit {
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
        if self.ui_mut().flowfield_show_arrow {
            for j in 0..GRID_SIZE {
                for i in 0..GRID_SIZE {
                    let (i, j) = (i as f32, j as f32);

                    let v = &flowfield.flow.get(&(i, j).into());
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
        Ok(())
    }
}
