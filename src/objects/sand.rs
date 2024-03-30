use crate::enums::cell_assets::import_assets;
use crate::enums::CellPhysicsType;
use crate::enums::CellPhysicsType::Tap;
use ecolor::Rgba;
use glam::{IVec2, Vec2, Vec3, Vec3Swizzles};
use hashbrown::HashMap;
use turborand::{rng::Rng, *};
use winit::event::VirtualKeyCode;

use crate::{
    enums::{cell_assets::CellAssets, CELL_SIZE, CHUNK_SIZE, CHUNK_SIZE_LEN},
    instance_data::InstanceData,
    world::WorldObject,
};

#[derive(Clone, Copy)]
pub struct Chunk {
    pub cells: [Option<(usize, Vec2)>; CHUNK_SIZE_LEN],
    pub cell_count: usize,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            cells: [None; CHUNK_SIZE_LEN],
            cell_count: 0,
        }
    }
}

impl Chunk {
    pub fn new_full(to_full: usize) -> Self {
        let mut cells = [None; CHUNK_SIZE_LEN];

        for i in CHUNK_SIZE_LEN / 2..CHUNK_SIZE_LEN - 1 {
            cells[i] = Some((0, Vec2::ZERO))
        }

        Self {
            cells,
            cell_count: 0,
        }
    }

    pub fn get(&self, pos: IVec2) -> Option<(usize, Vec2)> {
        match Chunk::ivec_to_vec_index(pos) {
            Some(index) => self.cells[index],
            None => None,
        }
    }

    pub fn get_mut(&mut self, pos: IVec2) -> Option<&mut Option<(usize, Vec2)>> {
        match Chunk::ivec_to_vec_index(pos) {
            Some(index) => Some(&mut self.cells[index]),
            None => None,
        }
    }

    pub fn insert(&mut self, pos: IVec2, cell: Option<(usize, Vec2)>) {
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
            // Handle edge case for index in the top row
            if index < CHUNK_SIZE.x as usize {
                return None;
            }

            // Calculate index upwards using integer division
            Some(index - CHUNK_SIZE.x as usize)
        } else {
            None
        }
    }

    pub fn global_pos_to_chunk_pos(global_pos: IVec2) -> IVec2 {
        // Adjust the global_pos for modulo operation to ensure positive results
        let mod_x = ((global_pos.x % CHUNK_SIZE.x) + CHUNK_SIZE.x) % CHUNK_SIZE.x;
        let mod_y = ((global_pos.y % CHUNK_SIZE.y) + CHUNK_SIZE.y) % CHUNK_SIZE.y;
        IVec2::new(mod_x, mod_y)
    }

    pub fn check_bounds(pos: IVec2) -> bool {
        Chunk::ivec_to_vec_index(pos).is_some()
    }
}

#[derive(Default)]
pub struct CellWorld {
    pub position: Vec2,
    pub chunks: HashMap<IVec2, Chunk>,
    pub chunk_count: i32,
    pub cell_assets_handles: CellAssets,
    pub rand: Rng,
    pub is_move: bool,
}

impl CellWorld {
    pub fn insert(&mut self, pos: IVec2, entity: Option<(usize, Vec2)>) {
        match self.get_mut_chunk(pos) {
            Some(chunk) => chunk.insert(Chunk::global_pos_to_chunk_pos(pos), entity),
            None => {
                let mut new_chunk = Chunk::default();
                new_chunk.insert(Chunk::global_pos_to_chunk_pos(pos), entity);
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

    pub fn get_mut(&mut self, pos: IVec2) -> Option<&mut Option<(usize, Vec2)>> {
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

    fn render_chunk(world: &CellWorld, chunk_pos: IVec2) -> Vec<InstanceData> {
        let mut material_data = vec![];
        let chunk_pos_local = (CellWorld::calculate_chunk_pos(chunk_pos) * CHUNK_SIZE).as_vec2();
        let chunk = world.get_chunk(chunk_pos);
        if let Some(chunk) = chunk {
            for y in 0..CHUNK_SIZE.y {
                for x in 0..CHUNK_SIZE.x {
                    let cell_pos = IVec2 { x: x, y: y };
                    if let Some(cell) = chunk.get(cell_pos) {
                        let color = match world.cell_assets_handles.get_color(cell.0) {
                            Some(color) => color,
                            None => Rgba::from_rgb(1.0, 0.0, 0.0),
                        };

                        material_data.push(InstanceData {
                            position: (cell_pos.as_vec2() + cell.1 + chunk_pos_local),
                            scale: 1.0,
                            color: color,
                        })
                    }
                }
            }
        }
        material_data
    }

    pub fn new() -> Self {
        let cell_assets_handles = import_assets().unwrap();

        let chunk = Chunk::new_full(0);

        let mut tap_chunk = Chunk::default();

        tap_chunk.cells[CHUNK_SIZE_LEN - 6] = Some((2, Vec2::ZERO));

        let mut chunks = HashMap::new();

        chunks.insert(IVec2::ZERO, chunk);
        chunks.insert(IVec2::new(-1, -1), chunk.clone());
        chunks.insert(IVec2::new(-1, 0), tap_chunk);

        Self {
            position: Vec2::ZERO,
            chunks,
            chunk_count: 1,
            cell_assets_handles,
            rand: Rng::new(),
            is_move: false,
        }
    }

    pub fn physics(&mut self) {
        if !self.is_move {
            return;
        }
        for (chunk_pos, chunk) in self.chunks.iter_mut() {
            // let mut chunk_below_option = world.get_mut_chunk(chunk_pos.clone() + IVec2{x:0, y:1});
            // let mut chunk_below = match chunk_below_option {
            //     Some(chunk_below) => chunk_below,
            //     None => {
            //         *chunk_below_option = Chunk::default()
            //     },
            // }
            let mut to_swap_list = vec![];
            let mut to_move_list = vec![];
            let mut to_insert_list = vec![];

            for i in 0..chunk.cells.len() - 1 {
                match chunk.cells[i] {
                    Some(cell) => match self
                        .cell_assets_handles
                        .assets_physics_behavior_vec
                        .get(cell.0)
                    {
                        Some(behavior) => match behavior {
                            CellPhysicsType::Sand => {
                                sand_physics(
                                    i,
                                    &chunk,
                                    &mut to_swap_list,
                                    &mut to_move_list,
                                    &mut self.rand,
                                );
                            }
                            CellPhysicsType::Fluid => {
                                fluid_physics(
                                    i,
                                    &chunk,
                                    &mut to_swap_list,
                                    &mut to_move_list,
                                    &mut self.rand,
                                );
                            }
                            CellPhysicsType::Tap(to_spawn) => {
                                tap_physics(
                                    &mut to_insert_list,
                                    i,
                                    chunk,
                                    &to_spawn,
                                    &self.cell_assets_handles,
                                    &mut self.rand,
                                );
                            }
                            CellPhysicsType::Solid => {}
                        },
                        None => {}
                    },
                    None => {}
                }
            }

            for to_swap in to_swap_list {
                chunk.cells.swap(to_swap.0, to_swap.1);
            }
            for to_move in to_move_list {
                if let Some(cell) = chunk.cells.get_mut(to_move.1) {
                    // Use get_mut for safe mutation
                    if let Some(cell) = cell {
                        // Use get_mut for safe mutation
                        cell.1 = cell.1 + to_move.0; // Explicit assignment
                    }
                }
            }
            for to_insert in to_insert_list {
                if let Some(cell) = chunk.cells.get_mut(to_insert.0) {
                    *cell = Some(to_insert.1);
                }
            }
        }
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
        let mut instance_data_vec = vec![];
        for (pos, _) in self.chunks.iter() {
            instance_data_vec.append(&mut Self::render_chunk(&self, pos.clone() * CHUNK_SIZE));
        }

        return instance_data_vec;

        // info!("{}", cells_material_data.0.len());
    }

    fn input(&mut self, delta_t: f32, event: &winit::event::WindowEvent, mouse_position: Vec2) {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                device_id,
                input,
                is_synthetic,
            } => match input.virtual_keycode {
                Some(code) => match code {
                    VirtualKeyCode::Q => {
                        self.is_move = true;
                    }
                    _ => {}
                },
                None => {}
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
            if chunk.cells[pos_below].is_none() {
                to_swap_list.push((i, pos_below));
                return;
            }
        } else {
            return;
        }
    };

    let is_none_below_left = get_is_none_by_offset(chunk, pos, IVec2 { x: -1, y: -1 });

    let is_none_below_right = get_is_none_by_offset(chunk, pos, IVec2 { x: 1, y: -1 });

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
    if let Some(cell_below_index) = Chunk::get_index_below(i) {
        if chunk.cells[cell_below_index].is_none() {
            if let Some(asset_id) = assets.get_index_by_name(to_spawn.to_string()) {
                to_insert_list.push((cell_below_index, (asset_id, Vec2::ZERO)))
            }
        }
    };
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
            if chunk.cells[pos_below].is_none() {
                to_swap_list.push((i, pos_below));
                return;
            }
        } else {
            return;
        }
    };

    let is_none_below_left = get_is_none_by_offset(chunk, pos, IVec2 { x: -1, y: -1 });

    let is_none_below_right = get_is_none_by_offset(chunk, pos, IVec2 { x: 1, y: -1 });

    move_if_none(
        to_swap_list,
        is_none_below_left,
        is_none_below_right,
        i,
        rand,
    );

    let is_none_left = get_is_none_by_offset(chunk, pos, IVec2 { x: -1, y: 0 });

    let is_none_right = get_is_none_by_offset(chunk, pos, IVec2 { x: 1, y: 0 });

    move_if_none(to_swap_list, is_none_left, is_none_right, i, rand)
}

fn get_is_none_by_offset(chunk: &Chunk, pos: IVec2, offset: IVec2) -> Option<usize> {
    let mut pos_offset = pos;
    pos_offset += offset;

    if let Some(cell) = Chunk::ivec_to_vec_index(pos_offset) {
        if chunk.cells[cell].is_none() {
            Some(cell)
        } else {
            None
        }
    } else {
        None
    }
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
