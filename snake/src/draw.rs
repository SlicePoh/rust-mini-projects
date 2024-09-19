use piston_window::types::Color;
use piston_window::{rectangle, Context, G2d, Transformed};

const BLOCK_SIZE: f64 = 25.0;

pub fn to_coord(game_coord: i32) -> f64 {
    (game_coord as f64) * BLOCK_SIZE
}

pub fn to_coord_u32(game_coord: i32) -> u32 {
    to_coord(game_coord) as u32
}

pub fn draw_block(color: Color, x: i32, y: i32, con: &Context, g: &mut G2d) {
    let gui_x = to_coord(x);
    let gui_y = to_coord(y);

    rectangle(
        color,
        [gui_x, gui_y, BLOCK_SIZE, BLOCK_SIZE],
        con.transform,
        g,
    );
}

pub fn draw_rectangle(
    color: Color,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    con: &Context,
    g: &mut G2d,
) {
    let rx = to_coord(x);
    let ry = to_coord(y);

    rectangle(
        color,
        [
            rx,
            ry,
            BLOCK_SIZE * (width as f64),
            BLOCK_SIZE * (height as f64),
        ],
        con.transform,
        g,
    );
}

pub fn draw_rotated_rectangle(
    color: Color,
    x: i32,
    y: i32,
    width: f32,
    height: f32,
    clockwise: bool,
    con: &Context,
    g: &mut G2d,
) {
    let gui_x = to_coord(x);
    let gui_y = to_coord(y);

    let rotation_angle = if clockwise {
        std::f64::consts::FRAC_PI_4 // 45 degrees clockwise
    } else {
        -std::f64::consts::FRAC_PI_4 // 45 degrees counterclockwise
    };

    rectangle(
        color,
        [
            gui_x,
            gui_y,
            BLOCK_SIZE * (width as f64),
            BLOCK_SIZE * (height as f64),
        ],
        con.transform
            .trans(gui_x, gui_y)
            .rot_rad(rotation_angle)
            .trans(-gui_x, -gui_y),
        g,
    );
}
