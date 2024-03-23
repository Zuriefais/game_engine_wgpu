use crate::Vertex;

pub const VERTICES: &[Vertex] = &[
    // Changed
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
    }, // A
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
    }, // B
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
    }, // C
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
    }, // D
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
    }, // E
];

pub const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];
