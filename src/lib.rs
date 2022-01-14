#[cfg(feature = "buddy-alloc")]
mod alloc;
mod wasm4;

use wasm4::*;

const CURSOR_SIZE: u8 = 4;
const MOUSE_CURSOR: [u8; 2] = [0b10010110, 0b01101001];
// SCREEN_SIZE 160x160
const CENTER: Point = Point::new(80.0, 80.0);
// rect, oval, line, text
const COLORS: u8 = 4;
static mut FRAME_COUNT: u32 = 0;
static mut WORLD: World = World::new();

#[no_mangle]
fn start() {
    let circle = Circle::new(CENTER, 20., 0, 255);

    unsafe {
        //*PALETTE = [0x9775a6, 0x683a68, 0x412752, 0x2d162c];
        *PALETTE = [0x2d162c, 0x412752, 0x683a68, 0x9775a6];
        *DRAW_COLORS = 0x4321;
        WORLD.add(circle);
    }
}

#[no_mangle]
fn update() {
    let time = unsafe { FRAME_COUNT as f32 / 60. };
    let mut World = unsafe { WORLD.clone() };
    let mut random = unsafe { oorandom::Rand32::new(FRAME_COUNT.into()) };
    let gamepad = unsafe { *GAMEPAD1 };
    if gamepad & BUTTON_1 != 0 {
        unsafe { *DRAW_COLORS = 4 }
    }

    let mouse = unsafe { (*MOUSE_X, *MOUSE_Y) };
    let mouse_pressed = unsafe { *MOUSE_BUTTONS & MOUSE_LEFT };
    let mouse_pressed_right = unsafe { *MOUSE_BUTTONS & MOUSE_RIGHT };

    if mouse_pressed != 0 {}

    //World.update();
    World.move_origin(Point::new(0.2, 0.2));
    for x in 0..SCREEN_SIZE {
        for y in 0..SCREEN_SIZE {
            let brightness = y;
            let max_brightness = SCREEN_SIZE;
            draw_pixel(x, y, brightness, max_brightness);
        }
    }
    let gradient = Gradient::new(Rectangle::new(Point::new(20., 0.), 100., 30.), 160, 200);
    gradient.draw();

    let circle2 = Circle::new(CENTER, 40. + time * 10., 0, 255);
    circle2.draw();
    World.draw();
    {
        let offset = (CURSOR_SIZE / 2) as i16;

        let x: i32 = (mouse.0 - offset).into();
        let y: i32 = (mouse.1 - offset).into();
        blit(
            &MOUSE_CURSOR,
            x,
            y,
            CURSOR_SIZE.into(),
            CURSOR_SIZE.into(),
            BLIT_1BPP,
        );
    }
    unsafe {
        FRAME_COUNT += 1;
        WORLD = World;
    }
}

fn index(x: u32, y: u32) -> usize {
    (x + y * SCREEN_SIZE) as usize
}

const BAYER2X2: [u32; 4] = [1, 3, 4, 2];
const CLUSTERED: [u32; 9] = [8, 3, 4, 6, 1, 2, 7, 5, 9];
const DISPERSED: [u32; 9] = [1, 7, 4, 5, 8, 3, 6, 2, 9];
const BAYER4X4: [u32; 16] = [1, 9, 3, 11, 13, 5, 15, 7, 4, 12, 2, 10, 16, 8, 14, 6];

fn draw_pixel(x: u32, y: u32, brightness: u32, max_brightness: u32) {
    // Additional color options.
    let step = max_brightness / (COLORS - 1) as u32;
    let bayer_size = 9;

    let color_base = brightness / step;
    if brightness > step * 3 {
        unsafe {
            *DRAW_COLORS = 1 + color_base as u16;
        }
        pixel(x, y);
    } else {
        // Get the value from the bayer matrix.
        let idx = ((x - y * 3) % bayer_size) as usize;
        let bayer = DISPERSED[idx] * step / bayer_size;
        let diff = brightness % step;
        let add_color = if diff > bayer { 1 } else { 0 };
        let draw_color = color_base + add_color;
        let draw_color = 1 + draw_color as u16;

        unsafe {
            *DRAW_COLORS = draw_color;
        }
        pixel(x, y);
    }
}

fn pixel(x: u32, y: u32) {
    // The byte index into the framebuffer that contains (x, y)
    let idx = (y as usize * 160 + x as usize) >> 2;

    // Calculate the bits within the byte that corresponds to our position
    let shift = (x as u8 & 0b11) << 1;
    let mask = 0b11 << shift;

    unsafe {
        let palette_color: u8 = (*DRAW_COLORS & 0xf) as u8;
        if palette_color == 0 {
            // Transparent
            return;
        }
        let color = (palette_color - 1) & 0b11;

        let framebuffer = FRAMEBUFFER.as_mut().expect("fb ref");

        framebuffer[idx] = (color << shift) | (framebuffer[idx] & !mask);
    }
}

fn line2(l: Vector) {
    line(
        l.start.x as i32,
        l.start.y as i32,
        l.end.x as i32,
        l.end.y as i32,
    );
    line(
        l.start.x as i32 + 1,
        l.start.y as i32,
        l.end.x as i32 + 1,
        l.end.y as i32,
    );
    line(
        l.start.x as i32,
        l.start.y as i32 + 1,
        l.end.x as i32,
        l.end.y as i32 + 1,
    );
}

fn circle(pos: &Point, radius: i32) {
    oval(
        (pos.x - radius as f32 / 2.0) as i32,
        (pos.y - radius as f32 / 2.0) as i32,
        radius.try_into().unwrap(),
        radius.try_into().unwrap(),
    );
}

#[derive(Clone)]
struct World {
    origin: Point,
    circles: Vec<Circle>,
}

impl World {
    const fn new() -> Self {
        Self {
            origin: Point::new(0., 0.),
            circles: Vec::new(),
        }
    }
    fn add(&mut self, circle: Circle) {
        self.circles.push(circle);
    }
    fn update(&mut self) {
        todo!();
    }
    fn move_origin(&mut self, delta: Point) {
        self.origin = self.origin + delta;
        for circle in &mut self.circles {
            circle.origin = circle.origin + delta;
        }
    }
    fn draw(&self) {
        for circle in &self.circles {
            circle.draw();
        }
    }
}

#[derive(Clone, Copy)]
struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn distance(&self, other: &Point) -> f32 {
        let delta_x = self.x - other.x;
        let delta_y = self.y - other.y;
        (delta_x * delta_x + delta_y * delta_y).sqrt()
    }
}

impl std::ops::Add for Point {
    type Output = Point;
    fn add(self, other: Point) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

struct Rectangle {
    pub origin: Point,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    const fn new(origin: Point, width: f32, height: f32) -> Self {
        Self {
            origin,
            width,
            height,
        }
    }
}

struct Gradient {
    pub extents: Rectangle,
    pub color_first: u32,
    pub color_second: u32,
}

impl Gradient {
    const fn new(extents: Rectangle, color_first: u32, color_second: u32) -> Self {
        Self {
            extents,
            color_first,
            color_second,
        }
    }

    pub fn draw(&self) {
        for rel_y in 0..self.extents.height as u32 {
            for rel_x in 0..self.extents.width as u32 {
                let x = rel_x + self.extents.origin.x as u32;
                let y = rel_y + self.extents.origin.y as u32;
                let brightness = map(
                    rel_y as f32,
                    0.0,
                    self.extents.height,
                    self.color_first as f32,
                    self.color_second as f32,
                );
                draw_pixel(x, y, brightness as u32, 255);
            }
        }
    }
}

#[derive(Clone)]
struct Circle {
    pub origin: Point,
    pub radius: f32,
    pub color_center: u32,
    pub color_end: u32,
}

impl Circle {
    fn new(origin: Point, radius: f32, color_center: u32, color_end: u32) -> Self {
        Self {
            origin,
            radius,
            color_center,
            color_end,
        }
    }
    fn draw_simple(&self) {
        circle(&self.origin, self.radius as i32);
    }

    fn draw(&self) {
        for screen_y in 0..SCREEN_SIZE {
            for screen_x in 0..SCREEN_SIZE {
                let screen_point = Point::new(screen_x as f32, screen_y as f32);
                let distance = self.origin.distance(&screen_point);
                if distance > self.radius {
                    continue;
                }
                let color = map(
                    distance,
                    0.0,
                    self.radius,
                    self.color_center as f32,
                    self.color_end as f32,
                );
                draw_pixel(screen_x, screen_y, color as u32, 255);
            }
        }
    }
}

fn map(value: f32, start1: f32, stop1: f32, start2: f32, stop2: f32) -> f32 {
    (value - start1) / (stop1 - start1) * (stop2 - start2) + start2
}

fn norm(value: f32, start: f32, stop: f32) -> f32 {
    map(value, start, stop, 0., 1.)
}

struct Vector {
    pub start: Point,
    pub end: Point,
}

impl Vector {
    const fn new(start: Point, end: Point) -> Self {
        Self { start, end }
    }
}
