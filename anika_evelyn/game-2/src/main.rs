// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine::wgpu;
use engine::gamestate::GameState;
use engine::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::time::{Duration, Instant};
const world_W: f32 = 320.0;
const world_H: f32 = 240.0;
const W: f32 = 160.0;
const H: f32 = 80.0;
const GUY_SPEED: f32 = 0.75;
const CATCH_DISTANCE: f32 = 9.0;
const TIME_LIMIT: u64 = 120;

struct Guy {
    pos: Vec2,
    direction: usize, // 0: front, 1: back, 2: left, 3: right
}

struct Potion {
    pos: Vec2,
    collected: bool,
    color: usize,
}

struct Spellbook {
    pos: Vec2,
    collected: bool,
    color: usize,
}

struct Game {
    camera: engine::Camera,
    guy: Guy,
    potions: Vec<Potion>,
    books: Vec<Spellbook>,
    start_timer: Option<Instant>,
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
            screen_size: [W, H],
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
        // 2: bgTitle, 3: bgBearAttack, 4: bgInstructions, 5: Win, 6: Lose

        // add background group
        let background_img = image::open("content-2/tile_floor.jpeg").unwrap().into_rgba8();
        let background_tex = engine.renderer.gpu.create_texture(
            //createarraytexture
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

        let guy = Guy {
            pos: Vec2 {
                x: world_W/2.0,
                y: 24.0,
            },
            direction: 0,
        };

        let mut rng = rand::thread_rng();
        let potions: Vec<Potion> = (0..10)
        .map(|_| Potion {pos: new_random_pos(), collected: false, color: rng.gen_range(0..2)})
        .collect();

        let books: Vec<Spellbook> = (0..4)
        .map(|_| Spellbook {pos: new_random_pos(), collected: false, color: rng.gen_range(0..2)})
        .collect();
        

        let font = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 0, 512, 0, 80, 8),
            10,
        );

        Game {
            camera,
            guy,
            potions: potions,
            books: books,
            start_timer: None,
            potions_collected: 0,
            font,
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
                self.state = GameState::Play;
                self.start_timer = Some(Instant::now());
            }
            return;
        }

        else if self.state == GameState::BearAttacked{
            return;
        }

        else if self.state == GameState::Win{
            return;
        }

        else if self.state == GameState::Lose{
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
            self.guy.pos.y = 2.5;
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

            // check guy collision with book 
            if let Some(idx) = self
            .books
            .iter()
            .position(|book| book.pos.distance(self.guy.pos) <= CATCH_DISTANCE)
            {
                if !self.books[idx].collected {
                    self.books[idx].collected = true;
                    println!("got book");

                    // TODO: add code
                }
            }
        }

        // currently win if have 5 potions
        if self.potions_collected >= 5 {
            println!("you win!");
            self.state = GameState::Win;
        }

        // timer for game
        if let Some(start_time) = self.start_timer {
            if new_now.duration_since(start_time) >= Duration::from_secs(TIME_LIMIT){
                self.state = GameState::Lose;
                self.start_timer = None;
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
            let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(2);
            trfs[0] = Transform::zeroed();
            uvs[0] = SheetRegion::zeroed();

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
            .upload_sprites(&engine.renderer.gpu, 2, 0..1);

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

        else if self.state == GameState::BearAttacked{
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
            let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(0);
            trfs[0] = Transform::zeroed();
            uvs[0] = SheetRegion::zeroed();

            // remove all other sprites
            let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(1);
            for i in 0..15 {
                trfs[i] = Transform::zeroed();
                uvs[i] = SheetRegion::zeroed();
            }

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 3, 0..1);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 0, 0..1);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 1, 0..15);

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
            let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(0);
            trfs[0] = Transform::zeroed();
            uvs[0] = SheetRegion::zeroed();

            // remove all other sprites
            let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(1);
            for i in 0..15 {
                trfs[i] = Transform::zeroed();
                uvs[i] = SheetRegion::zeroed();
            }

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 5, 0..1);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 0, 0..1);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 1, 0..15);

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
            let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(0);
            trfs[0] = Transform::zeroed();
            uvs[0] = SheetRegion::zeroed();

            // remove all other sprites
            let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(1);
            for i in 0..40 {
                trfs[i] = Transform::zeroed();
                uvs[i] = SheetRegion::zeroed();
            }

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 6, 0..1);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 0, 0..1);

            engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 1, 0..15);

            engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera);
            return;
        }

        // remove instructions bg
        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(4);
        trfs[0] = Transform::zeroed();
        uvs[0] = SheetRegion::zeroed();

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
        for i in 1..11 {
            if self.potions[i-1].collected { 
                trfs[i] = Transform::zeroed();
                uvs[i] = SheetRegion::zeroed();
            } else {
                trfs[i] = AABB {
                    center: self.potions[i-1].pos,
                    size: Vec2 { x: 9.6, y: 12.0 },
                }.into();
                uvs[i] = SheetRegion::new(0, 178, 1, 2, 96, 120);
            }
        }

        // set spellbooks
        for i in 11..15 {
            if self.books[i-11].collected { 
                trfs[i] = Transform::zeroed();
                uvs[i] = SheetRegion::zeroed();
            } else {
                trfs[i] = AABB {
                    center: self.books[i-11].pos,
                    size: Vec2 { x: 11.2, y: 12.0 },
                }.into();
                uvs[i] = SheetRegion::new(0, 276, 1, 4, 112, 120);
            }
        }

        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 0, 0..1);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 1, 0..15);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 4, 0..1);
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
