extern crate piston_window;

extern crate specs;
#[macro_use]
extern crate specs_derive;

use piston_window::*;
use std::time::Instant;
use std::time::Duration;

use specs::{Builder, World, DenseVecStorage, System, ReadStorage, Join, RunNow};

const WINDOW_HEIGHT: u32 = 800;
const WINDOW_WIDTH: u32 = 640;
const WINDOW_DIMENSIONS: [u32; 2] = [WINDOW_WIDTH, WINDOW_HEIGHT];

const RECT_WIDTH: f64 = 100.0;
const RECT_HEIGHT: f64 = 100.0;

// components

#[derive(Component, Debug)] // "Component" is from specs: automates the implementation of a component's storage
struct Position {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
struct Dimensions {
    width: f64, 
    height: f64,
} 

// systems

struct Slider;

impl<'a> System<'a> for Slider {
    type SystemData = (ReadStorage<'a, Position>,
                       ReadStorage<'a, Dimensions>);

    fn run(&mut self, (pos, dim): Self::SystemData) {
        for (pos, dim) in (&pos, &dim).join() {
            println!("Hello, {:?}", &dim);
            println!("Hello, {:?}", &pos);
        }
    }
}


fn main() {
    let mut world = init_world();
    let mut slider_sys = Slider;
    slider_sys.run_now(&world.res);
    
    //demo();
}

fn init_world() -> World {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Dimensions>();
    let rect = world.create_entity()
                .with(Position{ x: 0.0, y: 0.0})
                .with(Dimensions{ width: RECT_WIDTH, height: RECT_HEIGHT})
                .build();
    world
}

fn demo() {
    let mut window: PistonWindow = {
        WindowSettings::new("First Window", WINDOW_DIMENSIONS)
            .exit_on_esc(true) // Hitting escape exits the game.
            .build()
            .unwrap()
    };

    // Initial time.
    let mut time_before =  Instant::now();

    // Initial state of the rect. (all values in pixels).
    // X and Y denote upper left hand side of the rectangle.
    // [x, y, width, height]
    let mut rect = [0.0, 0.0, RECT_WIDTH, RECT_HEIGHT];
    let rect_color = [1.0, 0.0, 0.0, 1.0];

    println!("Running the demo!");
    while let Some(event) = window.next() {
        let elapsed_time = time_before.elapsed();
        time_before = Instant::now();
        slide_rect_down(&mut rect, elapsed_time);

        // pattern matching on user input
        match event.press_args() {
            Some(Button::Keyboard(Key::Right)) => move_rect_right(&mut rect),
            Some(Button::Keyboard(Key::Left)) => move_rect_left(&mut rect),
            _ => {},
        }
        
        //update the graphics window
        window.draw_2d(&event, |context, graphics| {
            // Clear all current drawings from the canvas.
            clear([1.0; 4], graphics);

            // Update the coordinates of the rectangle.
            rectangle(rect_color, rect, context.transform, graphics)
        });
    } 
}

fn slide_rect_down(rect: &mut [f64; 4], elapsed_time: Duration) {
    // pixels per second
    let speed = 200.0;
    let elapsed_nanos = elapsed_time.subsec_nanos();
    let nanoseconds_in_a_second = 1000000000.0;
    let y_delta = (elapsed_nanos as f64) * speed / nanoseconds_in_a_second;

    rect[1] = (rect[1] + y_delta) % (WINDOW_HEIGHT as f64);
}

///moves the rectangle to the right by RECT_WIDTH # of pixels
fn move_rect_right(rect: &mut [f64; 4]) {
    rect[0] = rect[0] + RECT_WIDTH;
    let window_width: f64 = WINDOW_WIDTH.into(); // converts to f64 for arithmetic
    if rect[0] > (window_width - RECT_WIDTH) {
        rect[0] = 0.0;
    }
}

///moves the rectangle to the left by RECT_WIDTH # of pixels
fn move_rect_left(rect: &mut[f64; 4]) {
    rect[0] = rect[0] - RECT_WIDTH;
    let window_width: f64 = WINDOW_WIDTH.into(); 
    if rect[0] < 0.0     {
        rect[0] = window_width - RECT_WIDTH;
    }
}
