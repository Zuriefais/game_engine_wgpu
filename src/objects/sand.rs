use crate::enums::cell_assets::import_assets;
use crate::enums::CellPhysicsType;
use crate::enums::CellPhysicsType::Tap;
use ecolor::Rgba;
use glam::{IVec2, Vec2, Vec3, Vec3Swizzles};
use hashbrown::HashMap;
use log::info;
use rayon::{prelude::*, vec};
use turborand::{rng::Rng, *};
use winit::event::{ElementState, MouseButton, VirtualKeyCode};

use crate::{
    enums::{cell_assets::CellAssets, CELL_SIZE, CHUNK_SIZE, CHUNK_SIZE_LEN},
    instance_data::InstanceData,
    world::WorldObject,
};

#[derive(Clone, Copy)]
pub struct Chunk {
    pub cells: [(usize, Vec2); CHUNK_SIZE_LEN],
    pub cell_count: usize,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            cells: [(0, Vec2::ZERO); CHUNK_SIZE_LEN],
            cell_count: 0,
        }
    }
}

impl Chunk {
    pub fn new_full(to_full: usize) -> Self {
        let mut cells = [(0, Vec2::ZERO); CHUNK_SIZE_LEN];

        for i in CHUNK_SIZE_LEN / 2..CHUNK_SIZE_LEN - 1 {
            cells[i] = (0, Vec2::ZERO)
        }

        Self {
            cells,
            cell_count: 0,
        }
    }

    pub fn get(&self, pos: IVec2) -> Option<(usize, Vec2)> {
        match Chunk::ivec_to_vec_index(pos) {
            Some(index) => Some(self.cells[index]),
            None => None,
        }
    }

    pub fn get_mut(&mut self, pos: IVec2) -> Option<&mut (usize, Vec2)> {
        match Chunk::ivec_to_vec_index(pos) {
            Some(index) => Some(&mut self.cells[index]),
            None => None,
        }
    }

    pub fn insert(&mut self, pos: IVec2, cell: (usize, Vec2)) {
        match self.get_mut(pos) {
            Some(some_cell) => {
                *some_cell = cell;
            }
            None => {}
        }
    }

    pub fn ivec_to_vec_index(pos: IVec2) -> Option<usize> {
        if pos.x >= 0 && pos.x < CHUNK_SIZE.x && pos.y >= 0 && pos.y < CHUNK_SIZE.y {
            Some((pos.y * CHUNK_SIZE.x + pos.x) as usize)
        } else {
            None
        }
    }

    pub fn vec_index_to_ivec(index: usize) -> Option<IVec2> {
        if index < (CHUNK_SIZE.x * CHUNK_SIZE.y) as usize {
            let y = index / CHUNK_SIZE.y as usize;
            let x = index % CHUNK_SIZE.x as usize;
            Some(IVec2 {
                x: x as i32,
                y: y as i32,
            })
        } else {
            None
        }
    }

    pub fn get_index_below(index: usize) -> Option<usize> {
        if index >= CHUNK_SIZE.x as usize {
            if index < CHUNK_SIZE.x as usize {
                return None;
            }

            Some(index - CHUNK_SIZE.y as usize)
        } else {
            None
        }
    }

    pub fn global_pos_to_chunk_pos(global_pos: IVec2) -> IVec2 {
        let mod_x = ((global_pos.x % CHUNK_SIZE.x) + CHUNK_SIZE.x) % CHUNK_SIZE.x;
        let mod_y = ((global_pos.y % CHUNK_SIZE.y) + CHUNK_SIZE.y) % CHUNK_SIZE.y;
        IVec2::new(mod_x, mod_y)
    }

    pub fn check_bounds(pos: IVec2) -> bool {
        Chunk::ivec_to_vec_index(pos).is_some()
    }

    fn render(&self, chunk_pos: IVec2) -> Vec<InstanceData> {
        let mut material_data = vec![];
        let chunk_pos_local = (CellWorld::calculate_chunk_pos(chunk_pos) * CHUNK_SIZE).as_vec2();

        for y in 0..CHUNK_SIZE.y {
            for x in 0..CHUNK_SIZE.x {
                let cell_pos = IVec2 { x, y };
                if let Some(cell) = self.get(cell_pos) {
                    if cell.0 != 0 {
                        material_data.push(InstanceData {
                            position: (cell_pos.as_vec2() + cell.1 + chunk_pos_local),
                            color: (cell.0 - 1) as u32,
                        })
                    }
                }
            }
        }
        material_data
    }
}

#[derive(Default)]
pub struct CellWorld {
    pub position: Vec2,
    pub chunks: HashMap<IVec2, Chunk>,
    pub chunk_count: i32,
    pub assets: CellAssets,
    pub rand: Rng,
    pub is_move: bool,
    pub selected: u32,
}

impl CellWorld {
    pub fn insert(&mut self, pos: IVec2, cell: (usize, Vec2)) {
        match self.get_mut_chunk(pos) {
            Some(chunk) => chunk.insert(Chunk::global_pos_to_chunk_pos(pos), cell),
            None => {
                let mut new_chunk = Chunk::default();
                new_chunk.insert(Chunk::global_pos_to_chunk_pos(pos), cell);
                self.chunks
                    .insert(CellWorld::calculate_chunk_pos(pos), new_chunk);
                self.chunk_count += 1;
            }
        }
    }

    pub fn get_mut_chunk(&mut self, pos: IVec2) -> Option<&mut Chunk> {
        self.chunks.get_mut(&CellWorld::calculate_chunk_pos(pos))
    }

    // pub fn get_mut_or_create_chunk(&mut self, pos: IVec2) -> &mut Chunk {

    // }

    pub fn get_chunk(&self, pos: IVec2) -> Option<&Chunk> {
        self.chunks.get(&CellWorld::calculate_chunk_pos(pos))
    }

    pub fn is_cell_empty(&self, pos: IVec2) -> bool {
        self.get(pos).is_none()
    }

    pub fn get(&self, pos: IVec2) -> Option<(usize, Vec2)> {
        let chunk_pos = CellWorld::calculate_chunk_pos(pos);
        self.chunks
            .get(&chunk_pos)
            .and_then(|chunk| chunk.get(Chunk::global_pos_to_chunk_pos(pos)))
    }

    pub fn get_mut(&mut self, pos: IVec2) -> Option<&mut (usize, Vec2)> {
        let chunk_pos = CellWorld::calculate_chunk_pos(pos);
        self.chunks.get_mut(&chunk_pos)?.get_mut(pos % CHUNK_SIZE)
    }

    pub fn calculate_chunk_pos(pos: IVec2) -> IVec2 {
        // Adjust the position before division to handle negative coordinates correctly
        let div_x = if pos.x < 0 {
            (pos.x + 1 - CHUNK_SIZE.x) / CHUNK_SIZE.x
        } else {
            pos.x / CHUNK_SIZE.x
        };
        let div_y = if pos.y < 0 {
            (pos.y + 1 - CHUNK_SIZE.y) / CHUNK_SIZE.y
        } else {
            pos.y / CHUNK_SIZE.y
        };
        IVec2::new(div_x, div_y)
    }

    pub fn new(assets: CellAssets) -> Self {
        let chunk = Chunk::new_full(0);

        let mut tap_chunk = Chunk::default();

        for i in 25..75 {
            tap_chunk.cells[CHUNK_SIZE_LEN - i] = (3, Vec2::ZERO);
        }

        let mut chunks = HashMap::new();

        chunks.insert(IVec2::ZERO, chunk);
        chunks.insert(IVec2::new(-1, -1), chunk.clone());

        for x in -5..5 {
            for y in -5..5 {
                chunks.insert(IVec2::new(x, y), tap_chunk);
            }
        }

        Self {
            position: Vec2::ZERO,
            chunks,
            chunk_count: 1,
            assets,
            rand: Rng::new(),
            is_move: false,
            selected: 1,
        }
    }

    pub fn select_cell_type(&mut self, keycode: VirtualKeyCode, state: ElementState) {
        match (keycode, state) {
            (code, state) => match (code, state) {
                (VirtualKeyCode::R, ElementState::Released) => {
                    self.selected = if self.assets.get((self.selected - 1) as usize).is_some() {
                        info!(
                            "selected {:?}",
                            self.assets.get((self.selected - 1) as usize)
                        );
                        self.selected - 1
                    } else {
                        self.selected
                    }
                }
                (VirtualKeyCode::T, ElementState::Released) => {
                    self.selected = if self.assets.get((self.selected + 1) as usize).is_some() {
                        info!(
                            "selected {:?}",
                            self.assets.get((self.selected + 1) as usize)
                        );
                        self.selected + 1
                    } else {
                        self.selected
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    pub fn physics(&mut self) {
        if !self.is_move {
            return;
        }
        for (chunk_pos, chunk) in self.chunks.iter_mut() {
            let mut to_swap_list: Vec<_> = vec![];
            let mut to_move_list = vec![];
            let mut to_insert_list = vec![];

            for i in 0..chunk.cells.len() - 1 {
                cell_physics(
                    &mut to_swap_list,
                    &mut to_insert_list,
                    &mut to_move_list,
                    i,
                    chunk,
                    &self.assets,
                    &mut self.rand,
                )
            }

            for (i, j) in to_swap_list {
                chunk.cells.swap(i, j);
            }
            for to_move in to_move_list {
                if let Some(cell) = chunk.cells.get_mut(to_move.1) {
                    cell.1 = cell.1 + to_move.0;
                }
            }
            for to_insert in to_insert_list {
                if let Some(cell) = chunk.cells.get_mut(to_insert.0) {
                    *cell = to_insert.1;
                }
            }
        }
    }
}

fn cell_physics(
    mut to_swap_list: &mut Vec<(usize, usize)>,
    mut to_insert_list: &mut Vec<(usize, (usize, Vec2))>,
    mut to_move_list: &mut Vec<(Vec2, usize)>,
    i: usize,
    chunk: &Chunk,
    assets: &CellAssets,
    mut rand: &mut Rng,
) {
    let cell = chunk.cells[i];
    if cell.0 == 0 {
        return;
    };
    match assets.get(chunk.cells[i].0 - 1) {
        Some(behavior) => match behavior.physics_behavior {
            CellPhysicsType::Sand => {
                sand_physics(i, &chunk, &mut to_swap_list, &mut to_move_list, &mut rand);
            }
            CellPhysicsType::Fluid => {
                fluid_physics(i, &chunk, &mut to_swap_list, &mut to_move_list, &mut rand);
            }
            CellPhysicsType::Tap(to_spawn) => {
                tap_physics(&mut to_insert_list, i, chunk, &to_spawn, assets, &mut rand);
            }
            CellPhysicsType::Solid => {}
        },
        None => {}
    }
}

impl WorldObject for CellWorld {
    fn get_pos(&self) -> Vec2 {
        self.position
    }

    fn set_pos(&mut self, pos: Vec2) {
        self.position = pos;
    }

    fn render(&self) -> Vec<InstanceData> {
        let pos_and_chunks = self.chunks.iter().enumerate();

        let instance_data_vec = pos_and_chunks
            .into_iter()
            .map(|(pos, chunk)| (pos, chunk.clone())) // Clone to avoid borrowing issues
            .par_bridge() // Use par_bridge to enable parallel processing
            .map(|(index, chunk)| chunk.1.render(*chunk.0 * CHUNK_SIZE))
            .collect::<Vec<_>>();
        let mut data = vec![];
        for mut arr in instance_data_vec {
            data.append(&mut arr)
        }
        return data;
    }

    fn input(&mut self, delta_t: f32, event: &winit::event::WindowEvent, mouse_position: Vec2) {
        match event {
            winit::event::WindowEvent::KeyboardInput { input, .. } => {
                match (input.virtual_keycode, input.state) {
                    (Some(code), state) => {
                        match (code, state) {
                            (VirtualKeyCode::Q, ElementState::Released) => {
                                self.is_move = !self.is_move;
                            }
                            _ => {}
                        }
                        self.select_cell_type(code, state)
                    }
                    _ => {}
                }
            }
            winit::event::WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                (ElementState::Pressed, MouseButton::Left) => self.insert(
                    mouse_position.as_ivec2(),
                    (self.selected as usize, Vec2::ZERO),
                ),
                (ElementState::Pressed, MouseButton::Right) => {
                    self.insert(mouse_position.as_ivec2(), (0, Vec2::ZERO))
                }
                _ => {}
            },
            _ => {}
        }
    }

    fn update(&mut self, delta_t: f32) {
        self.physics()
    }

    fn get_name(&self) -> String {
        "cell world".to_string()
    }
}

fn sand_physics(
    i: usize,
    chunk: &Chunk,
    to_swap_list: &mut Vec<(usize, usize)>,
    to_move_list: &mut Vec<(Vec2, usize)>,
    rand: &mut Rng,
) {
    let pos = Chunk::vec_index_to_ivec(i).unwrap();

    {
        if let Some(pos_below) = Chunk::get_index_below(i) {
            if chunk.cells[pos_below].0 == 0 {
                to_swap_list.push((i, pos_below));
                return;
            }
        } else {
            return;
        }
    };

    let is_none_below_left = get_is_none_below_left(chunk, i);

    let is_none_below_right = get_is_none_below_right(chunk, i);

    move_if_none(
        to_swap_list,
        is_none_below_left,
        is_none_below_right,
        i,
        rand,
    );
}

fn tap_physics(
    to_insert_list: &mut Vec<(usize, (usize, Vec2))>,
    i: usize,
    chunk: &Chunk,
    to_spawn: &String,
    assets: &CellAssets,
    rand: &mut Rng,
) {
    if let Some(i_below) = get_is_none_below(chunk, i) {
        if let Some(asset_id) = assets.get_index_by_name(to_spawn.to_string()) {
            to_insert_list.push((i_below, (asset_id + 1, Vec2::ZERO)));
        }
    }
}

fn fluid_physics(
    i: usize,
    chunk: &Chunk,
    to_swap_list: &mut Vec<(usize, usize)>,
    to_move_list: &mut Vec<(Vec2, usize)>,
    rand: &mut Rng,
) {
    let pos = Chunk::vec_index_to_ivec(i).unwrap();

    {
        if let Some(pos_below) = Chunk::get_index_below(i) {
            if chunk.cells[pos_below].0 == 0 {
                to_swap_list.push((i, pos_below));
                return;
            }
        } else {
            return;
        }
    };

    let is_none_below_left = get_is_none_below_left(chunk, i);

    let is_none_below_right = get_is_none_below_right(chunk, i);

    move_if_none(
        to_swap_list,
        is_none_below_left,
        is_none_below_right,
        i,
        rand,
    );

    let is_none_left = get_is_none_left(chunk, i);

    let is_none_right = get_is_none_right(chunk, i);

    move_if_none(to_swap_list, is_none_left, is_none_right, i, rand)
}

fn get_is_none_by_offset_vec2(chunk: &Chunk, pos: IVec2, offset: IVec2) -> Option<usize> {
    let mut pos_offset = pos;
    pos_offset += offset;

    if let Some(cell) = Chunk::ivec_to_vec_index(pos_offset) {
        if chunk.cells[cell].0 == 0 {
            Some(cell)
        } else {
            None
        }
    } else {
        None
    }
}

fn get_is_none_below(chunk: &Chunk, i: usize) -> Option<usize> {
    let i_below = i - CHUNK_SIZE.y as usize;
    if let Some(cell) = chunk.cells.get(i_below) {
        if cell.0 == 0 {
            return Some(i_below);
        }
    }
    None
}

fn get_is_none_left(chunk: &Chunk, i: usize) -> Option<usize> {
    let i_below = i - 1;
    if let Some(cell) = chunk.cells.get(i_below) {
        if cell.0 == 0 {
            return Some(i_below);
        }
    }
    None
}

fn get_is_none_right(chunk: &Chunk, i: usize) -> Option<usize> {
    let i_below = i + 1;
    if let Some(cell) = chunk.cells.get(i_below) {
        if cell.0 == 0 {
            return Some(i_below);
        }
    }
    None
}

fn get_is_none_up(chunk: &Chunk, i: usize) -> Option<usize> {
    let i_below = i + CHUNK_SIZE.y as usize;
    if let Some(cell) = chunk.cells.get(i_below) {
        if cell.0 == 0 {
            return Some(i_below);
        }
    }
    None
}

fn get_is_none_below_right(chunk: &Chunk, i: usize) -> Option<usize> {
    let i_below = i - CHUNK_SIZE.y as usize - 1;
    if let Some(cell) = chunk.cells.get(i_below) {
        if cell.0 == 0 {
            return Some(i_below);
        }
    }
    None
}

fn get_is_none_below_left(chunk: &Chunk, i: usize) -> Option<usize> {
    let i_below = i - CHUNK_SIZE.y as usize + 1;
    if let Some(cell) = chunk.cells.get(i_below) {
        if cell.0 == 0 {
            return Some(i_below);
        }
    }
    None
}

fn get_is_none_up_right(chunk: &Chunk, i: usize) -> Option<usize> {
    let i_below = i + CHUNK_SIZE.y as usize - 1;
    if let Some(cell) = chunk.cells.get(i_below) {
        if cell.0 == 0 {
            return Some(i_below);
        }
    }
    None
}

fn get_is_none_up_left(chunk: &Chunk, i: usize) -> Option<usize> {
    let i_below = i + CHUNK_SIZE.y as usize + 1;
    if let Some(cell) = chunk.cells.get(i_below) {
        if cell.0 == 0 {
            return Some(i_below);
        }
    }
    None
}

fn move_if_none(
    to_swap_list: &mut Vec<(usize, usize)>,
    is_none: Option<usize>,
    is_none1: Option<usize>,
    i: usize,
    rand: &mut Rng,
) {
    match (is_none, is_none1) {
        (None, None) => {}
        (None, Some(cell)) => {
            to_swap_list.push((cell, i));
            return;
        }
        (Some(cell), None) => {
            to_swap_list.push((cell, i));
            return;
        }
        (Some(cell), Some(cell2)) => {
            if rand.bool() {
                to_swap_list.push((cell, i))
            } else {
                to_swap_list.push((cell2, i))
            }
            return;
        }
    }
}
