// TODO: use AABB instead of Rect for centered box, so collision checking doesn't have to offset by half size

use engine::wgpu;
use engine::{geom::*, Camera, Engine, SheetRegion, Transform, Zeroable};
use rand::Rng;
const world_W: f32 = 320.0;
const world_H: f32 = 240.0;
const W: f32 = 75.0;
const H: f32 = 50.0;
const GUY_SPEED: f32 = 1.0;
const SPRITE_MAX: usize = 16;
const CATCH_DISTANCE: f32 = 16.0;
const COLLISION_STEPS: usize = 3;
struct Guy {
    pos: Vec2,
}

struct Log {
    pos: Vec2,
}

struct Bear {
    pos: Vec2,
}

struct Tree {
    pos: Vec2,
}

struct Game {
    camera: engine::Camera,
    trees: Vec<Tree>,
    guy: Guy,
    bears: Vec<Bear>,
    logs: Vec<Log>,
    apple_timer: u32,
    score: u32,
    font: engine::BitFont,
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
        // SPRITE GROUPS: 0: bg, 1: man, 2: bear, 3: log, 4: trees

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
        let sprite_img = image::open("content/man_sheet.png").unwrap().into_rgba8();
        let sprite_tex = engine.renderer.gpu.create_texture(
            &sprite_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            sprite_img.dimensions(),
            Some("spr-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &sprite_tex,
            vec![Transform::zeroed(); 1], // just one man
            vec![SheetRegion::zeroed(); 1],
            camera,
        );

        // add bear group
        let bear_img = image::open("content/bear_sheet.png").unwrap().into_rgba8();
        let bear_tex = engine.renderer.gpu.create_texture(
            &bear_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            bear_img.dimensions(),
            Some("bear-demo.png"),
        );
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &bear_tex,
            vec![Transform::zeroed(); 2], // 2 bears
            vec![SheetRegion::zeroed(); 2],
            camera,
        );

        let setting_img = image::open("content/setting_sheet.png").unwrap().into_rgba8();
        let setting_tex = engine.renderer.gpu.create_texture(
            &setting_img,
            wgpu::TextureFormat::Rgba8UnormSrgb,
            setting_img.dimensions(),
            Some("setting-demo.png"),
        );

        // add log group
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &setting_tex,
            vec![Transform::zeroed(); SPRITE_MAX],
            vec![SheetRegion::zeroed(); SPRITE_MAX],
            camera,
        );

        // add tree group
        engine.renderer.sprites.add_sprite_group(
            &engine.renderer.gpu,
            &setting_tex,
            vec![Transform::zeroed(); SPRITE_MAX],
            vec![SheetRegion::zeroed(); SPRITE_MAX],
            camera,
        );

        let guy = Guy {
            pos: Vec2 {
                x: world_W/2.0,
                y: 24.0,
            },
        };

        let mut rng = rand::thread_rng();
        let trees: Vec<Tree> = (0..16)
        .map(|_| Tree {pos: Vec2 {x: rng.gen_range(0.0..world_W-1.0), y: rng.gen_range(0.0..world_H-1.0)}})
        .collect();

        let logs: Vec<Log> = (0..16)
        .map(|_| Log {pos: Vec2 {x: rng.gen_range(0.0..world_W-1.0), y: rng.gen_range(0.0..world_H-1.0)}})
        .collect();

        let bears: Vec<Bear> = (0..2)
        .map(|_| Bear {pos: Vec2 {x: rng.gen_range(0.0..world_W), y: rng.gen_range(0.0..world_H)}})
        .collect();
        

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
            apple_timer: 0,
            score: 0,
            font,
        }
    }
    fn update(&mut self, engine: &mut Engine) {
        //TBD: can be put in char_actions
        let xdir = engine.input.key_axis(engine::Key::Left, engine::Key::Right);
        self.guy.pos.x += xdir * GUY_SPEED;
        let ydir = engine.input.key_axis(engine::Key::Down, engine::Key::Up);
        self.guy.pos.y += ydir * GUY_SPEED;
        
        // TODO: move bears
        let mut rng = rand::thread_rng();
        for (bear, i) in self.bears.iter_mut().zip(0..2) {
            let xdir = if rng.gen_range(0..2) > 0 {1.0} else {-1.0};
            let ydir = if rng.gen_range(0..2) > 0 {1.0} else {-1.0};
            bear.pos.x += xdir * 2.0;
            bear.pos.y += ydir * 2.0;
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
        }

        // TODO: check tree collision
       
        // TODO: Pick Up Log
        

    }
    fn render(&mut self, engine: &mut Engine) {
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
        uvs_bg[0] = SheetRegion::new(0, 0, 0, 16, 1920, 1280);

        // set guy
        let (trfs, uvs) = engine.renderer.sprites.get_sprites_mut(1);
        trfs[0] = AABB {
            center: self.guy.pos,
            size: Vec2 { x: 12.0, y: 16.5 },
        }
        .into();
        uvs[0] = SheetRegion::new(0, 177, 1, 1, 166, 232);

        // set bears
        let (trfs_bears, uvs_bears) = engine.renderer.sprites.get_sprites_mut(2);
        for i in 0..2 {
            trfs_bears[i] = AABB {
                center: self.bears[i].pos,
                size: Vec2 { x: 12.0, y: 16.5 },
            }.into();
            uvs_bears[i] = SheetRegion::new(0, 211, 1, 1, 200, 270);
        }
        // set logs
        let (trfs_log, uvs_log) = engine.renderer.sprites.get_sprites_mut(3);
        for i in 0..16 {
            trfs_log[i] = AABB {
                center: self.logs[i].pos,
                size: Vec2 { x: 9.0, y: 4.0 },
            }.into();
            uvs_log[i] = SheetRegion::new(0, 809, 1, 1, 672, 224);
        }
        // set trees
        let (trfs_tree, uvs_tree) = engine.renderer.sprites.get_sprites_mut(4);
        for i in 0..16 {
            trfs_tree[i] = AABB {
                center: self.trees[i].pos,
                size: Vec2 { x: 16.0, y: 16.0 },
            }.into();
            uvs_tree[i] = SheetRegion::new(0, 809, 227, 3, 294, 294);
        }

        let score_str = self.score.to_string();
        let text_len = score_str.len();

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
            .upload_sprites(&engine.renderer.gpu, 1, 0..1);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 2, 0..2);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 3, 0..16);
        engine
            .renderer
            .sprites
            .upload_sprites(&engine.renderer.gpu, 4, 0..16);
        // engine
        //     .renderer
        //     .sprites
        //     .set_camera_all(&engine.renderer.gpu, self.camera);
        self.camera.screen_pos = [self.guy.pos.x-(W/2.0), self.guy.pos.y-(H/2.0)];
        engine
            .renderer
            .sprites
            .set_camera_all(&engine.renderer.gpu, self.camera)
    }
}
fn main() {
    Engine::new(winit::window::WindowBuilder::new()).run::<Game>();
}
