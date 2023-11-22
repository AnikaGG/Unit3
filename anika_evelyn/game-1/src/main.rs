// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine::wgpu;
use engine::animation::Animation;
use engine::gamestate::GameState;
use engine::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
use std::time::{Duration, Instant};
const world_W: f32 = 320.0;
const world_H: f32 = 240.0;
const W: f32 = 40.39;
const H: f32 = 20.06;
const GUY_SPEED: f32 = 0.75;
const SPRITE_MAX: usize = 16;
const CATCH_DISTANCE: f32 = 3.0;
const BEAR_DISTANCE: f32 = 4.0;
const COLLISION_STEPS: usize = 2;
const FIREPIT_POS: Vec2 = Vec2 {x: world_W/2.0 - 10.0, y: 24.0};
const TIME_LIMIT: u64 = 120;

struct Guy {
    pos: Vec2,
    log_idx: usize, // this idx referes to logs[idx-1], so 0 means no log
    direction: usize, // 0: front, 1: back, 2: left, 3: right
}

struct Log {
    pos: Vec2,
    collected: bool,
}

struct Bear {
    pos: Vec2,
    bear_count: u32,
}

struct Game {
    camera: engine::Camera,
    trees: Vec<AABB>,
    guy: Guy,
    bears: Vec<Bear>,
    logs: Vec<Log>,
    start_timer: Option<Instant>,
    logs_collected: u32,
    font: engine::BitFont,
    bear_anim: Animation,
    state: GameState,
    has_fire: bool,
    friction_count: u32,
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
        let background_img = image::open("content/background_grass.jpeg").unwrap().into_rgba8();
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
        let sprite_img = image::open("content/spritesheet.png").unwrap().into_rgba8();
        let sprite_tex = engine.renderer.gpu.create_texture(
            &sprite_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            sprite_img.dimensions(),
            Some("spr-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &sprite_tex,
            vec![Transform::zeroed(); 40], 
            vec![SheetRegion::zeroed(); 40], // man (0), bears (1-4), logs (5-20), trees (21-36), campsite (37), firepit (38), fire (39)
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
            log_idx: 0,
            direction: 0,
        };

        let mut rng = rand::thread_rng();
        let trees: Vec<AABB> = (0..16)
        .map(|_| AABB {
            center: Vec2 { x: rng.gen_range(0.0..world_W-1.0), y: rng.gen_range(0.0..world_H-1.0) },
            size: Vec2 { x: 11.0, y: 11.0 }})
        .collect();

        let logs: Vec<Log> = (0..16)
        .map(|_| Log {pos: Vec2 {x: rng.gen_range(0.0..world_W-1.0), y: rng.gen_range(0.0..world_H-1.0)}, collected: false})
        .collect();

        let bears: Vec<Bear> = (0..4)
        .map(|_| Bear {pos: Vec2 {x: rng.gen_range(0.0..world_W), y: rng.gen_range(0.0..world_H)}, bear_count: 0})
        .collect();

        // print bears coords on one line
        for bear in bears.iter() {
            println!("bear: {} {} ", bear.pos.x, bear.pos.y);
        }

        // Create the bear animation
        let mut bear_frames: Vec<[f32; 6]> = vec![
            // bear 5 positions
            [0.0, 973.0, 1.0, 2.0, 64.0, 33.0],
            [0.0, 1039.0, 1.0, 2.0, 64.0, 33.0],
            [0.0, 1105.0, 1.0, 2.0, 64.0, 33.0],
            [0.0, 973.0, 36.0, 2.0, 64.0, 33.0],
            [0.0, 1039.0, 36.0, 2.0, 64.0, 33.0],
        ];
        let mut bear_anim = Animation {
            states: bear_frames,
            frame_counter: 0,
            rate: 40,
            state_number: 0,
            is_facing_left: false,
            sprite_width: 64.0,
            is_looping: true,
            is_done: false,
        };
        

        let font = engine::BitFont::with_sheet_region(
            '0'..='9',
            SheetRegion::new(0, 0, 512, 0, 80, 8),
            10,
        );
        Game {
            camera,
            guy,
            trees: trees,
            logs: logs,
            bears: bears,
            start_timer: None,
            logs_collected: 0,
            font,
            bear_anim,
            state: GameState::Title,
            has_fire: false,
            friction_count: 0,
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

        let mut contacts = Vec::with_capacity(self.trees.len());
        // TODO: for multiple guys this might be better as flags on the guy for what side he's currently colliding with stuff on
        for _iter in 0..COLLISION_STEPS {
            let guy_aabb = AABB {
                center: self.guy.pos,
                size: Vec2 { x: 16.0, y: 16.0 },
            };
            contacts.clear();
            // TODO: to generalize to multiple guys, need to iterate over guys first and have guy_index, rect_index, displacement in a contact tuple
            contacts.extend(
                self.trees
                    .iter()
                    .enumerate()
                    .filter_map(|(ri, w)| w.displacement(guy_aabb).map(|d| (ri, d))),
            );
            if contacts.is_empty() {
                break;
            }
            // This part stays mostly the same for multiple guys, except the shape of contacts is different
            contacts.sort_by(|(_r1i, d1), (_r2i, d2)| {
                d2.length_squared()
                    .partial_cmp(&d1.length_squared())
                    .unwrap()
            });
            for (wall_idx, _disp) in contacts.iter() {
                // TODO: for multiple guys should access self.guys[guy_idx].
                let guy_aabb = AABB {
                    center: self.guy.pos,
                    size: Vec2 { x: 4.0, y: 4.0 },
                };
                let wall = self.trees[*wall_idx];
                let mut disp = wall.displacement(guy_aabb).unwrap_or(Vec2::ZERO);
                // We got to a basically zero collision amount
                if disp.x.abs() < std::f32::EPSILON || disp.y.abs() < std::f32::EPSILON {
                    break;
                }
                // Guy is left of wall, push left
                if self.guy.pos.x < wall.center.x {
                    disp.x *= -1.0;
                }
                // Guy is below wall, push down
                if self.guy.pos.y < wall.center.y {
                    disp.y *= -1.0;
                }
                if disp.x.abs() <= disp.y.abs() {
                    self.guy.pos.x += disp.x;
                    // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                } else if disp.y.abs() <= disp.x.abs() {
                    self.guy.pos.y += disp.y;
                    // so far it seems resolved; for multiple guys this should probably set a flag on the guy
                }
            }
        }

        // campsite collision
        let guy_aabb = AABB {
            center: self.guy.pos,
            size: Vec2 { x: 4.0, y: 4.0 },
        };
        let campsite_aabb: AABB = AABB {
            center: Vec2 {
                x: world_W / 2.0 + 10.0,
                y: 24.0,
            },
            size: Vec2 { x: 10.0, y: 13.6 },
        }.into();
        let mut disp = campsite_aabb.displacement(guy_aabb).unwrap_or(Vec2::ZERO);
        // Guy is left of wall, push left
        if self.guy.pos.x < campsite_aabb.center.x {
            disp.x *= -1.0;
        }
        // Guy is below wall, push down
        if self.guy.pos.y < campsite_aabb.center.y {
            disp.y *= -1.0;
        }
        if disp.x.abs() <= disp.y.abs() {
            self.guy.pos.x += disp.x;
        } else if disp.y.abs() <= disp.x.abs() {
            self.guy.pos.y += disp.y;
        }

        //TBD: can be put in char_actions
        // keep guy in frame
        // check for guy collision with tree
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

        // move log with guy
        if self.guy.log_idx > 0 {
            self.logs[self.guy.log_idx-1].pos.x = self.guy.pos.x;
            self.logs[self.guy.log_idx-1].pos.y = self.guy.pos.y - 2.0;
        }

        // TODO: move bears
        let mut rng = rand::thread_rng();
        for (bear, i) in self.bears.iter_mut().zip(0..4) {
            if bear.bear_count == 4 {
                let xdir = if rng.gen_range(0..2) > 0 {1.0} else {-1.0};
                let ydir = if rng.gen_range(0..2) > 0 {1.0} else {-1.0};
                bear.pos.x += xdir * 1.0;
                bear.pos.y += ydir * 1.0;
                bear.bear_count =0;
            }
            else {
                bear.bear_count+=1;
            }
            // keep bear in frame
            if bear.pos.x >= world_W {
                bear.pos.x = world_W - 1.0;
            }
            if bear.pos.x <= 0.0 {
                bear.pos.x = 1.0;
            }
            if bear.pos.y >= world_H {
                bear.pos.y = world_H - 1.0;
            }
            if bear.pos.y <= 0.0 {
                bear.pos.y = 1.0;
            }

            // Set bear animation frames
            let current_state = self.bear_anim.get_current_state();
        }

        // check guy collision with log
        if self.guy.log_idx == 0 {
            if let Some(idx) = self
            .logs
            .iter()
            .position(|log| log.pos.distance(self.guy.pos) <= CATCH_DISTANCE)
            {
                if !self.logs[idx].collected {
                    self.logs[idx].collected = true;
                    self.guy.log_idx = idx + 1;
                    println!("got log");
                }
            }
        }

        //TBD: press space to release log???
        // check guy collision with firepit to release log
        if self.guy.log_idx > 0 {
            if FIREPIT_POS.distance(self.guy.pos) <= CATCH_DISTANCE+2.0 {
                if engine.input.is_key_pressed(winit::event::VirtualKeyCode::Return) {
                    self.logs[self.guy.log_idx-1].collected = true;
                    self.guy.log_idx = 0;
                    self.logs_collected = self.logs_collected + 1;
                    println!("{} log in fire", self.logs_collected);
                }
            }
        }

        if FIREPIT_POS.distance(self.guy.pos) <= CATCH_DISTANCE+2.0 {
            if self.logs_collected >= 3 && !self.has_fire {
                if engine.input.is_key_pressed(winit::event::VirtualKeyCode::F) {
                    self.friction_count += 1;
                    println!("{}", self.friction_count);
                    println!("increase");
                }
                if self.friction_count > 7 {
                self.has_fire = true;
                println!("friction!");
                self.friction_count = 0;
                }
            }
        }

        // check guy collision with bear
        if self.bears.iter().any(|bear| bear.pos.distance(self.guy.pos) <= BEAR_DISTANCE) {
            self.state = GameState::BearAttacked;
        }

        // currently win if have 5 logs and fire
        if self.logs_collected == 5 && self.has_fire{
            self.state = GameState::Win;
        }

        // timer for game
        if let Some(start_time) = self.start_timer {
            if new_now.duration_since(start_time) >= Duration::from_secs(TIME_LIMIT) && !self.has_fire{
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
            for i in 0..40 {
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
            .upload_sprites(&engine.renderer.gpu, 1, 0..40);

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
            for i in 0..40 {
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
            .upload_sprites(&engine.renderer.gpu, 1, 0..40);

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
            .upload_sprites(&engine.renderer.gpu, 1, 0..40);

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
        uvs_bg[0] = SheetRegion::new(0, 0, 0, 6, 1920, 1280);

        // set sprites
        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(1);

        // 0: front, 1: back, 2: left, 3: right
        let front_sheet = SheetRegion::new(0, 1363, 227, 3, 166, 232);
        let back_sheet = SheetRegion::new(0, 1187, 227, 3, 174, 228);
        let left_sheet = SheetRegion::new(0, 1531, 227, 3, 156, 238);
        let right_sheet = SheetRegion::new(0, 1689, 227, 3, 154, 234);
        // set guy
        trfs[0] = AABB {
            center: self.guy.pos,
            size: Vec2 { x: 6.0, y: 8.25 },
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
        
        // SPRITE INDICES: man (0), bears (1-4), logs (5-20), trees (21-36), campsite (37), firepit (38), fire (39)

        // set bears
        for i in 1..5 {
            // Get the current state from the animation for each bear
            let current_state = self.bear_anim.get_current_state();

            // Use the current state for setting AABB and SheetRegion
            trfs[i] = AABB {
                center: self.bears[i - 1].pos,
                size: Vec2 { x: 16.0, y: 8.75 },
            }
            .into();
            uvs[i] = SheetRegion::new(
                current_state[0] as u16,
                current_state[1] as u16,
                current_state[2] as u16,
                current_state[3] as u16,
                current_state[4] as u16,
                current_state[5] as u16,
            );

            // Tick the animation for the next frame
            self.bear_anim.tick();
        }

        // set logs
        for i in 5..21 {
            trfs[i] = AABB {
                center: self.logs[i-5].pos,
                size: Vec2 { x: 6.0, y: 2.0 },
            }.into();
            uvs[i] = SheetRegion::new(0, 1171, 1, 2, 672, 224);
        }

        // set trees
        for i in 21..37 {
            trfs[i] = AABB {
                center: self.trees[i-21].center,
                size: Vec2 { x: 11.0, y: 11.0 },
            }.into();
            uvs[i] = SheetRegion::new(0, 1187, 463, 4, 294, 294);
        }

        // set campsite
        trfs[37] = AABB {
            center: Vec2 {
                x: world_W / 2.0 + 10.0,
                y: 24.0,
            },
            size: Vec2 { x: 10.0, y: 13.6 },
        }.into();
        uvs[37] = SheetRegion::new(0, 769, 759, 2, 230, 314);

        // set firepit
        trfs[38] = AABB {
            center: FIREPIT_POS,
            size: Vec2 { x: 10.0, y: 10.0},
        }.into();
        uvs[38] = SheetRegion::new(0, 1, 759, 4, 322, 322);

        // add fire
        let fire_size = if self.has_fire { Vec2 { x: 6.0, y: 6.1 } } else { Vec2 { x: 0.0, y: 0.0 } };
        trfs[39] = AABB {
            center: Vec2 {
                x: FIREPIT_POS.x,
                y: FIREPIT_POS.y,
            },
            size:  fire_size
        }.into();
        uvs[39] = SheetRegion::new(0, 811, 141, 1, 286, 292);

        // let score_str = self.score.to_string();
        // let text_len = score_str.len();

        // engine.renderer.sprites.resize_sprite_group(
        //     &engine.renderer.gpu,
        //     0,
        //     sprite_count + text_len,
        // );

        // self.font.draw_text(
        //     &mut engine.renderer.sprites,
        //     0,
        //     sprite_count,
        //     &score_str,
        //     Vec2 {
        //         x: 16.0,
        //         y: H - 16.0,
        //     }
        //     .into(),
        //     16.0,
        // );

        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 0, 0..1);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 1, 0..40);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 4, 0..1);
        // engine
        //     .renderer
        //     .sprites
        //     .set_camera_all(&engine.renderer.gpu, self.camera);
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
