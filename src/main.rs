extern crate piston_window;
extern crate specs;
#[macro_use]
extern crate specs_derive;

use piston_window::*;
use specs::prelude::*;
use std::time::Instant;

/****** Constants ******/

const WINDOW_HEIGHT: u32 = 800;
const WINDOW_WIDTH: u32 = 640;
const WINDOW_DIMENSIONS: [u32; 2] = [WINDOW_WIDTH, WINDOW_HEIGHT];
const RECT_WIDTH: f64 = (WINDOW_WIDTH as f64) / 8.0;
const RECT_HEIGHT: f64 = (WINDOW_HEIGHT as f64) / 10.0;
const NANOS_PER_SECOND: f64 = 1000000000.0;
const MAX_MOVE_SPEED: f64 = 0.05;
const MAX_SPAWN_SPEED: f64 = 0.5;

/****** Components ******/

#[derive(Component, Debug)]
struct Position {
    x: f64,
    y: f64,
}

#[derive(Component, Debug)]
struct Dimensions {
    width: f64,
    height: f64,
}

#[derive(Component, Debug)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

#[derive(Component, Debug)]
struct DropSpeed(f64);


#[derive(Component, Debug)]
struct Active(bool);

/****** Resources ******/
// These tend to be globals

struct Clock {
    start: Instant,
    last_player_move: Instant,
    last_drop: Instant,
    last_spawn: Instant,
}

struct KeysPressed {
    left: bool,
    right: bool,
    space: bool,
}

struct Actions {
    move_left: bool,
    move_right: bool,
    create_rect: bool,
}

/****** Systems ******/

struct Dropper;

impl<'a> System<'a> for Dropper {
    type SystemData = (
        WriteStorage<'a, Active>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Clock>,
        ReadStorage<'a, DropSpeed>,
        WriteExpect<'a, Actions>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut active, mut pos, mut clock, drop_speed, mut actions) = data;
        let time_since_drop = clock.last_drop.elapsed();

        for (active, pos, drop_speed) in (&mut active, &mut pos, &drop_speed).join() {
            // Only drop the active block.
            if active.0 {
                // drop rects down
                let y_delta = time_since_drop.subsec_nanos() as f64 * drop_speed.0 / NANOS_PER_SECOND;
                pos.y = (pos.y + y_delta) % (WINDOW_HEIGHT as f64);

                // Not sure why we would multiply RECT_HEIGHT by 3.0/2.0... but it works... :)
                // why does this happen?
                // somehow I fixed it by making seemingly unrelated changes... weird. 
                let y_max = (WINDOW_HEIGHT as f64) - (RECT_HEIGHT);

                if pos.y >= y_max {
                    // Block has hit bottom of screen.
                    pos.y = y_max; 
                    active.0 = false; 
                    actions.create_rect = true;
                } 
                clock.last_drop = Instant::now();
            }
        }   
    }
}

struct Movement;

impl<'a> System<'a> for Movement {
    type SystemData = (
        WriteStorage<'a, Active>,
        WriteStorage<'a, Position>,
        WriteExpect<'a, Clock>,
        WriteExpect<'a, Actions>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut active, mut pos, mut clock, mut actions) = data;

        let time_since_move = clock.last_player_move.elapsed();
        let secs_since_move =
            time_since_move.as_secs() as f64 + time_since_move.subsec_nanos() as f64 * 1e-9;

        for (active, pos) in (&mut active, &mut pos).join() {
            // Only move the active block
            if active.0 {
                let window_width: f64 = WINDOW_WIDTH.into();
                if actions.move_right {
                    if secs_since_move > MAX_MOVE_SPEED {
                        pos.x = pos.x + RECT_WIDTH;
                        if pos.x > (window_width - RECT_WIDTH) {
                            pos.x = 0.0;
                        }
                        clock.last_player_move = Instant::now();
                    }
                }
                if actions.move_left {
                    if secs_since_move > MAX_MOVE_SPEED {
                        pos.x = pos.x - RECT_HEIGHT;
                        if pos.x < 0.0 {
                            pos.x = window_width - RECT_WIDTH
                        }
                        clock.last_player_move = Instant::now();
                    }
                }
            }
        }
        actions.move_right = false;
        actions.move_left = false;
    }
}

struct RectSpawner;

impl<'a> System<'a> for RectSpawner {
    type SystemData = (
        Entities<'a>,
        WriteExpect<'a, Clock>,
        Read<'a, LazyUpdate>,
        WriteExpect<'a, Actions>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut clock, updater, mut actions) = data;

        let time_since_spawn = clock.last_spawn.elapsed();
        let secs_since_spawn =
            time_since_spawn.as_secs() as f64 + time_since_spawn.subsec_nanos() as f64 * 1e-9;

        if secs_since_spawn > MAX_SPAWN_SPEED {
            if actions.create_rect {
                let new_rect = entities.create();
                updater.insert(
                    new_rect,
                    Dimensions {
                        width: RECT_WIDTH,
                        height: RECT_HEIGHT,
                    },
                );
                updater.insert(new_rect, Position { x: 0.0, y: 0.0 });
                updater.insert(
                    new_rect,
                    Color {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    },
                );
                updater.insert(new_rect, DropSpeed(100.0));
                updater.insert(new_rect, Active(true));

                clock.last_spawn = Instant::now();
                actions.create_rect = false;
            }
        }
    }
}

struct Printer;

impl<'a> System<'a> for Printer {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Dimensions>,
        ReadStorage<'a, Color>,
    );

    fn run(&mut self, (pos, dim, color): Self::SystemData) {
        for (pos, dim, color) in (&pos, &dim, &color).join() {
            println!("Printer -> {:?}", &dim);
            println!("Printer -> {:?}", &pos);
            println!("Printer -> {:?}", &color);
        }
    }
}

struct Timer;

impl<'a> System<'a> for Timer {
    type SystemData = (WriteExpect<'a, Clock>);

    fn run(&mut self, time: Self::SystemData) {
        // impl
    }
}

struct Render {
    window: PistonWindow,
}

impl<'a> System<'a> for Render {
    type SystemData = (
        ReadStorage<'a, Position>,
        ReadStorage<'a, Dimensions>,
        ReadStorage<'a, Color>,
        WriteExpect<'a, KeysPressed>,
        WriteExpect<'a, Actions>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (pos, dim, color, mut keys, mut actions) = data;

        if let Some(event) = self.window.next() {
            // saving user Movement for process by other systems

            match event.press_args() {
                Some(Button::Keyboard(Key::Right)) => {
                    keys.right = true;
                    actions.move_right = true;
                }
                Some(Button::Keyboard(Key::Left)) => {
                    keys.left = true;
                    actions.move_left = true;
                }
                Some(Button::Keyboard(Key::Space)) => {
                    keys.space = true;
                    actions.create_rect = true;
                }
                _ => {}
            }

            match event.release_args() {
                Some(Button::Keyboard(Key::Right)) => keys.right = false,
                Some(Button::Keyboard(Key::Left)) => keys.left = false,
                Some(Button::Keyboard(Key::Space)) => keys.space = false,
                _ => {}
            }

            // updating graphics
            self.window.draw_2d(&event, |context, graphics| {
                clear([1.0; 4], graphics);

                //for all entities with pos, dim, and color properties (i.e. rect)
                for (pos, dim, color) in (&pos, &dim, &color).join() {
                    let temp_rect = [pos.x, pos.y, dim.width, dim.height];
                    let temp_color = [color.r, color.g, color.b, color.a];
                    rectangle(temp_color, temp_rect, context.transform, graphics);
                }
            });
        }
    }
}

/****** Main ******/

fn main() {
    ecs_demo();
}

fn ecs_demo() {
    let window = init_window();
    let mut world = init_world();

    let mut dispatcher = DispatcherBuilder::new()
        .with(Dropper, "dropper", &[])
        //.with(Printer, "Printer", &[]) // for debugging
        .with(Timer, "timer", &[])
        .with(RectSpawner, "spawner", &[]) 
        .with(Movement, "movement", &[])
        .with_thread_local(Render{window})
        .build();

    loop {
        // warning, esc will not close program, need to ctrl-c in CLI
        dispatcher.dispatch(&mut world.res);
        world.maintain();
    }
}

fn init_world() -> World {
    let mut world = World::new();
    world.register::<Position>();
    world.register::<Dimensions>();
    world.register::<Color>();
    world.register::<DropSpeed>();
    world.register::<Active>();

    world.add_resource(KeysPressed {
        left: false,
        right: false,
        space: false,
    });
    world.add_resource(Actions {
        move_left: false,
        move_right: false,
        create_rect: false,
    });
    world.add_resource(Clock {
        start: Instant::now(),
        last_player_move: Instant::now(),
        last_drop: Instant::now(),
        last_spawn: Instant::now(),
    });

    world
        .create_entity()
        .with(Position { x: 0.0, y: 0.0 })
        .with(Dimensions {
            width: RECT_WIDTH,
            height: RECT_HEIGHT,
        })
        .with(Color {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        })
        .with(DropSpeed(100.0))
        .with(Active(true))
        .build();

    world
}

fn init_window() -> PistonWindow {
    let window: PistonWindow = {
        WindowSettings::new("DoubleTet", WINDOW_DIMENSIONS)
            .exit_on_esc(true)
            .build()
            .unwrap()
    };
    window
}
