// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine::wgpu;
use engine::gamestate::GameState;
use engine::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::time::{Duration, Instant};
use std::usize;
const world_W: f32 = 320.0;
const world_H: f32 = 240.0;
const W: f32 = 160.0;
const H: f32 = 80.0;
const GUY_SPEED: f32 = 0.75;
const CATCH_DISTANCE: f32 = 9.0;
const TIME_LIMIT: u64 = 30;
const REMEMBER_TIME_LIMIT: u64 = 4;

struct Guy {
    pos: Vec2,
    direction: usize, // 0: front, 1: back, 2: left, 3: right
}

struct Potion {
    pos: Vec2,
    collected: bool,
    // 0: blue, 1: purple, 2: green, 3: red, 4: yellow
    color: usize,
}

struct Spellbook {
    pos: Vec2,
    collected: bool,
    // 0: good, 1: death
    color: usize,
}

struct Game {
    camera: engine::Camera,
    guy: Guy,
    potions: Vec<Potion>,
    level_potions: Vec<i32>,
    books: Vec<Spellbook>,
    level_timer: Option<Instant>,
    timer_length: usize,
    level: usize,
    total_time: u64,
    potions_collected: u32,
    font: engine::BitFont,
    state: GameState,
}

// function creates a new random position
fn new_random_pos() -> Vec2 {
    let mut rng = rand::thread_rng();
    return Vec2 {x: rng.gen_range(0.0..world_W-1.0), y: rng.gen_range(0.0..world_H-1.0)};
}


impl engine::Game for Game {
    fn new(engine: &mut Engine) -> Self {
        let camera = Camera {
            screen_pos: [0.0, 0.0],
            screen_size: [world_W, world_H],
        };
        #[cfg(target_arch = "wasm32")]
        let sprite_img = {
            let img_bytes = include_bytes!("content/demo.png");
            image::load_from_memory_with_format(&img_bytes, image::ImageFormat::Png)
                .map_err(|e| e.to_string())
                .unwrap()
                .into_rgba8()
        };

        #[cfg(not(target_arch = "wasm32"))]
        // SPRITE GROUPS: 0: bg, 1: sprites
        // 2: bgTitle, 3: bgBearAttack, 4: bgInstructions, 5: Win, 6: Lose, 7: font

        // add background group
        let background_img = image::open("content-2/tile_floor.jpeg").unwrap().into_rgba8();
        let background_tex = engine.renderer.gpu.create_texture(
            &background_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            background_img.dimensions(),
            Some("background-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &background_tex,
            vec![Transform::zeroed(); 1],
            vec![SheetRegion::zeroed(); 1],
            camera,
        );

        // add man group
        let sprite_img = image::open("content-2/spritesheet.png").unwrap().into_rgba8();
        let sprite_tex = engine.renderer.gpu.create_texture(
            &sprite_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            sprite_img.dimensions(),
            Some("spr-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &sprite_tex,
            vec![Transform::zeroed(); 15], 
            vec![SheetRegion::zeroed(); 15],  // man (0), potions (1-10), spellbooks (11-14)
            camera,
        );

        // add Title group
        let background_title_img = image::open("content/bgTitle.png").unwrap().into_rgba8();
        let background_title_tex = engine.renderer.gpu.create_texture(
            &background_title_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            background_title_img.dimensions(),
            Some("background-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &background_title_tex,
            vec![Transform::zeroed(); 1],
            vec![SheetRegion::zeroed(); 1],
            camera,
        );

        // add End Game Bear Attack group
        let background_bear_attack_img = image::open("content/bgBearAttack.png").unwrap().into_rgba8();
        let background_bear_attack_tex = engine.renderer.gpu.create_texture(
            &background_bear_attack_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            background_bear_attack_img.dimensions(),
            Some("background-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &background_bear_attack_tex,
            vec![Transform::zeroed(); 1],
            vec![SheetRegion::zeroed(); 1],
            camera,
        );

        // add Instructions group
        let background_instructions_img = image::open("content/campingInstructions.png").unwrap().into_rgba8();
        let background_instructions_tex = engine.renderer.gpu.create_texture(
            &background_instructions_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            background_instructions_img.dimensions(),
            Some("background-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &background_instructions_tex,
            vec![Transform::zeroed(); 1],
            vec![SheetRegion::zeroed(); 1],
            camera,
        );

        // add Win group
        let background_instructions_img = image::open("content/winFire.png").unwrap().into_rgba8();
        let background_instructions_tex = engine.renderer.gpu.create_texture(
            &background_instructions_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            background_instructions_img.dimensions(),
            Some("background-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &background_instructions_tex,
            vec![Transform::zeroed(); 1],
            vec![SheetRegion::zeroed(); 1],
            camera,
        );

        // add Lose group
        let background_instructions_img = image::open("content/Lose.jpg").unwrap().into_rgba8();
        let background_instructions_tex = engine.renderer.gpu.create_texture(
            &background_instructions_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            background_instructions_img.dimensions(),
            Some("background-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &background_instructions_tex,
            vec![Transform::zeroed(); 1],
            vec![SheetRegion::zeroed(); 1],
            camera,
        );

        // add font group
        let font_img = image::open("content-2/mario_numbers.png").unwrap().into_rgba8();
        let font_tex = engine.renderer.gpu.create_texture(
            &font_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            font_img.dimensions(),
            Some("font.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &font_tex,
            vec![Transform::zeroed(); 2],
            vec![SheetRegion::zeroed(); 2],
            camera,
        );

        let guy = Guy {
            pos: Vec2 {
                x: world_W/2.0,
                y: 24.0,
            },
            direction: 0,
        };

        // initialize random sequence
        let mut level_potions: Vec<i32> = Vec::new();
        for i in 0..5 {
            let mut rng = rand::thread_rng();
            level_potions.push(rng.gen_range(0..5));
        }

        let font = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 0, 20, 0, 900, 150),
            10,
        );

        Game {
            camera,
            guy,
            potions: Vec::new(),
            level_potions: level_potions,
            books: Vec::new(),
            level_timer: None,
            level: 1,
            timer_length: 0,
            total_time: TIME_LIMIT,
            potions_collected: 0,
            font: font,
            state: GameState::Title,
        }
    }
    fn update(&mut self, engine: &mut Engine) {

        if self.state == GameState::Title{
            if engine.input.is_key_pressed(winit::event::VirtualKeyCode::Space) {
                self.state = GameState::Instructions;
            }
            return;
        }

        else if self.state == GameState::Instructions{
            if engine.input.is_key_pressed(winit::event::VirtualKeyCode::Space) {
                self.state = GameState::ShowLevel;
                self.level_timer = Some(Instant::now());
            }
            return;
        }

        else if self.state == GameState::Win{
            return;
        }

        else if self.state == GameState::Lose{
            return;
        }

        else if self.state == GameState::ShowLevel {
            let mut new_now = Instant::now();
            // timer for game
            if let Some(timer) = self.level_timer {
                // start game play
                if new_now.duration_since(timer) >= Duration::from_secs(REMEMBER_TIME_LIMIT){
                    self.state = GameState::Play;
                    self.level_timer = Some(Instant::now());
                    self.total_time = TIME_LIMIT;

                    let mut rng = rand::thread_rng();
                    // reset random positions for 2 of each potion
                    self.potions = (0..10)
                    .map(|i| Potion {pos: new_random_pos(), collected: false, 
                        color: if i < 2 { 0 } 
                        else if i < 4 { 1 }
                        else if i < 6 { 2 }
                        else if i < 8 { 3 }
                        else if i < 10 { 4 }
                        else {0}})
                    .collect();
                    // reset random positions of 2 good and 2 death spellbook
                    self.books = (0..4)
                    .map(|i| Spellbook {pos: new_random_pos(), collected: false, color: if i < 2 { 0 } else { 1 },})
                    .collect();
                }
            }
            return;
        }

        let mut new_now = Instant::now();

        // keep guy in frame
        let xdir = engine.input.key_axis(engine::Key::Left, engine::Key::Right);
        if self.guy.pos.x >= world_W-2.0 {
            self.guy.pos.x = world_W - 2.5;
        }
        else if self.guy.pos.x <= 2.0 {
            self.guy.pos.x = 2.5;
        }
        else {
            self.guy.pos.x += xdir * GUY_SPEED;
        }
        let ydir = engine.input.key_axis(engine::Key::Down, engine::Key::Up);
        if self.guy.pos.y >= world_H-2.0 {
            self.guy.pos.y = world_H - 2.5;
        }
        else if self.guy.pos.y <= 2.0 {
            self.guy.pos.y = 5.0;
        }
        else {
            self.guy.pos.y += ydir * GUY_SPEED;
        }

        // 0: front, 1: back, 2: left, 3: right
        // update direction
        if engine.input.is_key_down(winit::event::VirtualKeyCode::Down) {
            self.guy.direction = 0;
        }
        else if engine.input.is_key_down(winit::event::VirtualKeyCode::Up) {
            self.guy.direction = 1;
        }
        else if engine.input.is_key_down(winit::event::VirtualKeyCode::Left) {
            self.guy.direction = 2;
        }
        else if engine.input.is_key_down(winit::event::VirtualKeyCode::Right) {
            self.guy.direction = 3;
        }

        // TODO: move spellbooks
        let mut rng = rand::thread_rng();
        for (book, i) in self.books.iter_mut().zip(0..4) {
            if !book.collected {
                book.pos.x = book.pos.x + rng.gen_range(-0.5..0.5);
                book.pos.y = book.pos.y + rng.gen_range(-0.5..0.5);

            }
            // keep book in frame
            if book.pos.x >= world_W {
                book.pos.x = world_W - 1.0;
            }
            if book.pos.x <= 0.0 {
                book.pos.x = 1.0;
            }
            if book.pos.y >= world_H {
                book.pos.y = world_H - 1.0;
            }
            if book.pos.y <= 0.0 {
                book.pos.y = 1.0;
            }
        }

        // check return pressed
        if engine.input.is_key_down(winit::event::VirtualKeyCode::Return) {

            // check collision with vial
            if let Some(idx) = self
            .potions
            .iter()
            .position(|potion| potion.pos.distance(self.guy.pos) <= CATCH_DISTANCE)
            {
                if !self.potions[idx].collected {
                    self.potions[idx].collected = true;
                    self.potions_collected += 1;
                    println!("got potion");

                    // TODO: add code
                }
            }

        }

        // check guy collision with book 
        if let Some(idx) = self
        .books
        .iter()
        .position(|book| book.pos.distance(self.guy.pos) <= CATCH_DISTANCE)
        {
            if !self.books[idx].collected {
                self.books[idx].collected = true;
                println!("got book");
                if self.books[idx].color== 0 {
                    // Adding 5 seconds to the timer
                    self.total_time += 5;
                }
                else {
                    // Removing 7 seconds to the timer
                    self.total_time -= 7;
                }
            }
        }

        // currently win if have 5 potions -> Show new level
        if self.potions_collected >= (self.level_potions.len()-1).try_into().unwrap() {
            println!("you won level!");
            println!("{}", self.level_potions.len()-1);

            self.state = GameState::ShowLevel;
            self.level_timer = Some(Instant::now());
            self.level += 1;
            let mut new_len = self.level_potions.len()+1;
            self.potions_collected = 0;
            // new potion sequence
            self.level_potions.clear();
            for i in 0..new_len {
                let mut rng = rand::thread_rng();
                self.level_potions.push(rng.gen_range(0..5));
            }

            if self.level == 11 {
                println!("you win!");
                self.state = GameState::Win;
                return;
            }
        }

        // timer for game
        if let Some(timer) = self.level_timer {
            if new_now.duration_since(timer) >= Duration::from_secs(TIME_LIMIT){
                self.state = GameState::Lose;
                self.level_timer = None;
            }
        }
        
    }

    fn render(&mut self, engine: &mut Engine) {

        if self.state == GameState::Title{
            // set bg image
            let (trfs_bg, uvs_bg) = engine.renderer.sprites.get_sprites_mut(2);
            trfs_bg[0] = AABB {
                center: Vec2 {
                    x: W / 2.0,
                    y: H / 2.0,
                },
                size: Vec2 { x: W, y: H },
            }
            .into();
            uvs_bg[0] = SheetRegion::new(0, 0, 0, 1, 533, 400);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 2, 0..1);

            engine
            .renderer
            .sprites
            .set_camera(&engine.renderer.gpu, 2, self.camera);
            return;
        }

        else if self.state == GameState::Instructions {
            // remove title bg
            remove_background(2, engine);

            // set bg image
            let (trfs_bg, uvs_bg) = engine.renderer.sprites.get_sprites_mut(4);
            trfs_bg[0] = AABB {
                center: Vec2 {
                    x: W / 2.0,
                    y: H / 2.0,
                },
                size: Vec2 { x: W, y: H },
            }
            .into();
            uvs_bg[0] = SheetRegion::new(0, 0, 0, 1, 533, 400);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 4, 0..1);

            engine
            .renderer
            .sprites
            .set_camera(&engine.renderer.gpu, 4, self.camera);
            return;
        }

        else if self.state == GameState::Attack{
            // set bg image
            let (trfs_bg, uvs_bg) = engine.renderer.sprites.get_sprites_mut(3);
            trfs_bg[0] = AABB {
                center: Vec2 {
                    x: self.camera.screen_pos[0] + W / 2.0,
                    y: self.camera.screen_pos[1] + H / 2.0,
                },
                size: Vec2 { x: W, y: H },
            }
            .into();
            uvs_bg[0] = SheetRegion::new(0, 0, 0, 1, 533, 400);

            // remove bg
            remove_background(0, engine);

            // remove all other sprites
            clear_sprites(engine, self.timer_length);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 3, 0..1);

            engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera);
            return;
        }

        else if self.state == GameState::Win{
            // set bg image
            let (trfs_bg, uvs_bg) = engine.renderer.sprites.get_sprites_mut(5);
            trfs_bg[0] = AABB {
                center: Vec2 {
                    x: self.camera.screen_pos[0] + W / 2.0,
                    y: self.camera.screen_pos[1] + H / 2.0,
                },
                size: Vec2 { x: W, y: H },
            }
            .into();
            uvs_bg[0] = SheetRegion::new(0, 0, 0, 1, 533, 400);

            // remove bg
            remove_background(0, engine);

            // remove all other sprites
            clear_sprites(engine, self.timer_length);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 5, 0..1);

            engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera);
            return;
        }

        else if self.state == GameState::Lose{
            // set bg image
            let (trfs_bg, uvs_bg) = engine.renderer.sprites.get_sprites_mut(6);
            trfs_bg[0] = AABB {
                center: Vec2 {
                    x: self.camera.screen_pos[0] + W / 2.0,
                    y: self.camera.screen_pos[1] + H / 2.0,
                },
                size: Vec2 { x: W, y: H },
            }
            .into();
            uvs_bg[0] = SheetRegion::new(0, 0, 0, 1, 533, 400);

            // remove bg
            remove_background(0, engine);

            // remove all other sprites
            clear_sprites(engine, self.timer_length);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 6, 0..1);

            engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera);
            return;
        }  else if self.state == GameState::ShowLevel { 

            // remove instructions bg
            remove_background(4, engine);

            // remove all other sprites
            clear_sprites(engine, self.timer_length);

            // set bg image
            let (trfs_bg, uvs_bg) = engine.renderer.sprites.get_sprites_mut(0);
            trfs_bg[0] = AABB {
                center: Vec2 {
                    x: world_W / 2.0,
                    y: world_H / 2.0,
                },
                size: Vec2 { x: world_W, y: world_H },
            }
            .into();
            uvs_bg[0] = SheetRegion::new(0, 0, 0, 6, 626, 416);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 0, 0..1);

            // add potion sequence to screen
            for i in 0..self.level_potions.len()-1 {
                let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(1);
                trfs[i] = AABB {
                    center: Vec2 {
                        x: self.camera.screen_pos[0] + ((W - 40.0) /(self.level_potions.len()-1) as f32 * i as f32) + 20.0,
                        y: self.camera.screen_pos[1] + H/2.0,
                    },
                    size: Vec2 { x: 9.6, y: 12.0 },
                }.into();
                if self.level_potions[i] == 0 {
                    uvs[i] = SheetRegion::new(0, 178, 1, 2, 96, 120);
                }
                else if self.level_potions[i] == 1 {
                    uvs[i] = SheetRegion::new(0, 440, 1, 2, 80, 120);
                }
                else if self.level_potions[i] == 2 {
                    uvs[i] = SheetRegion::new(0, 309, 123, 2, 96, 120);
                }
                else if self.level_potions[i] == 3 {
                    uvs[i] = SheetRegion::new(0, 407, 123, 2, 96, 120);
                }
                else if self.level_potions[i] == 4 {
                    uvs[i] = SheetRegion::new(0, 165, 417, 2, 96, 120);
                }
            }

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 1, 0..(self.level_potions.len()-1));

            return;
        }

        // set sprites
        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(1);

        // 0: front, 1: back, 2: left, 3: right
        let front_sheet = SheetRegion::new(0, 210, 123, 3, 97, 174);
        let back_sheet = SheetRegion::new(0, 1, 1, 3, 93, 174);
        let left_sheet = SheetRegion::new(0, 1, 245, 3, 86, 170);
        let right_sheet = SheetRegion::new(0, 407, 245, 3, 88, 170);

        // set guy
        trfs[0] = AABB {
            center: self.guy.pos,
            size: Vec2 { x: 8.9, y: 17.25 },
        }
        .into();
        if self.guy.direction == 0 {
            uvs[0] = front_sheet;
        }
        else if self.guy.direction == 1 {
            uvs[0] = back_sheet;
        }
        else if self.guy.direction == 2 {
            uvs[0] = left_sheet;
        }
        else {
            uvs[0] = right_sheet;
        }
        
        // SPRITE INDICES: man (0), potions (1-10), spellbooks (11-14)

        // set potions
        // 0: blue, 1: purple, 2: green, 3: red, 4: yellow
        for i in 1..11 {
            if self.potions[i-1].collected { 
                trfs[i] = Transform::zeroed();
                uvs[i] = SheetRegion::zeroed();
            } else {
                trfs[i] = AABB {
                    center: self.potions[i-1].pos,
                    size: Vec2 { x: 9.6, y: 12.0 },
                }.into();
                if self.potions[i-1].color == 0 {
                    uvs[i] = SheetRegion::new(0, 178, 1, 2, 96, 120);
                }
                else if self.potions[i-1].color == 1 {
                    uvs[i] = SheetRegion::new(0, 440, 1, 2, 80, 120);
                }
                else if self.potions[i-1].color == 2 {
                    uvs[i] = SheetRegion::new(0, 309, 123, 2, 96, 120);
                }
                else if self.potions[i-1].color == 3 {
                    uvs[i] = SheetRegion::new(0, 407, 123, 2, 96, 120);
                }
                else if self.potions[i-1].color == 4 {
                    uvs[i] = SheetRegion::new(0, 165, 417, 2, 96, 120);
                }
            }
        }

        // set good spellbook
        for i in 11..15 {
            if self.books[i-11].collected { 
                trfs[i] = Transform::zeroed();
                uvs[i] = SheetRegion::zeroed();
            } else {
                trfs[i] = AABB {
                    center: self.books[i-11].pos,
                    size: Vec2 { x: 11.2, y: 12.0 },
                }.into();
                if self.books[i-11].color == 0 {
                    uvs[i] = SheetRegion::new(0, 276, 1, 4, 112, 120);
                }
                else {
                    uvs[i] = SheetRegion::new(0, 96, 123, 4, 112, 120);
                }
            }
        }

        // set timer
        let time_remaining = self.total_time - self.level_timer.unwrap().elapsed().as_secs();
        let timer_str = time_remaining.to_string();
        self.timer_length = timer_str.len();
        engine.renderer.sprites.resize_sprite_group(
            &engine.renderer.gpu,
            7,
            self.timer_length,
        );
        self.font.draw_text(
            &mut engine.renderer.sprites,
            7,
            0,
            &timer_str,
            Vec2 { // put numbers in corner
                x: self.camera.screen_pos[0] + W / 2.0 - 70.0,
                y: self.camera.screen_pos[1] + H / 2.0 + 40.0,
            }
            .into(),
            8.0,
        );

        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 1, 0..15);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 7, 0..self.timer_length);
        self.camera.screen_pos = [
        (self.guy.pos.x - (W / 2.0)).max(0.0).min(world_W - self.camera.screen_size[0]),
        (self.guy.pos.y - (H / 2.0)).max(0.0).min(world_H - self.camera.screen_size[1]),
        ];
        engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera)
    }
}
fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}


fn clear_sprites(engine: &mut Engine, timer_len: usize) {

    // remove all other sprites
    let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(1);
    for i in 0..15 {
        trfs[i] = Transform::zeroed();
        uvs[i] = SheetRegion::zeroed();
    }

    engine
    .renderer
    .sprites
    .upload_sprites(&engine.renderer.gpu, 1, 0..15);

    // remove all fonts
    let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(7);
    for i in 0..timer_len {
        trfs[i] = Transform::zeroed();
        uvs[i] = SheetRegion::zeroed();
    }

    engine
    .renderer
    .sprites
    .upload_sprites(&engine.renderer.gpu, 7, 0..timer_len);


}

fn remove_background(background_group: usize, engine: &mut Engine) {

    let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(background_group);
    trfs[0] = Transform::zeroed();
    uvs[0] = SheetRegion::zeroed();

    engine
    .renderer
    .sprites
    .upload_sprites(&engine.renderer.gpu, background_group, 0..1);

}