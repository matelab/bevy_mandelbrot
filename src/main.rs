mod mandelbrot;

use bevy_clicking::{ClickEvent, ClickingPlugin, DoubleclickEvent};
use mandelbrot::{MandelbrotMaterial, MandelbrotMesh2dBundle, MandelbrotPlugin};

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    window::WindowResized,
};

struct Screen {
    width: f32,
    height: f32,
    aspect: f32,
}

type MousePos = Vec2;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(MandelbrotPlugin::default())
        .add_plugin(ClickingPlugin)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .insert_resource(Screen {
            width: 1.0,
            height: 1.0,
            aspect: 1.0,
        })
        .insert_resource(MousePos::new(0.0, 0.0))
        .add_startup_system(setup)
        .add_system(bevy::input::system::exit_on_esc_system)
        .add_system(bevy::input::mouse::mouse_button_input_system)
        .add_system(fractal_drag)
        .add_system(fractal_zoom)
        .add_system(fractal_start)
        .add_system(cursor_moved)
        .add_system(change_iters)
        .add_system(window_size)
        .add_system(reset_start)
        .add_system(reset_view)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MandelbrotMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    asset_server.watch_for_changes().unwrap();
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(MandelbrotMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::default())).into(),
        transform: Transform::default(),
        material: materials.add(MandelbrotMaterial {
            center: Vec2::new(-0.4, 0.0),
            start: Vec2::new(0.0, 0.0),
            scale: 0.4,
            aspect: 1.0,
            iters: 64,
        }),
        ..Default::default()
    });
}

fn window_size(
    mut size_event: EventReader<WindowResized>,
    mut query: Query<(&mut Transform, &Handle<MandelbrotMaterial>)>,
    mut mat: ResMut<Assets<MandelbrotMaterial>>,
    mut screen: ResMut<Screen>,
) {
    for wse in size_event.iter() {
        for (mut tr, handle) in query.iter_mut() {
            tr.scale = Vec3::new(wse.width as f32, wse.height as f32, 1.0);
            mat.get_mut(handle).unwrap().aspect = wse.width as f32 / wse.height as f32;
            screen.width = wse.width as f32;
            screen.height = wse.height as f32;
            screen.aspect = screen.width / screen.height;
        }
    }
}

fn fractal_drag(
    mut mouse_event: EventReader<MouseMotion>,
    mut query: Query<&Handle<MandelbrotMaterial>>,
    screen: Res<Screen>,
    lmb: Res<Input<MouseButton>>,
    mut mats: ResMut<Assets<MandelbrotMaterial>>,
) {
    for ev in mouse_event.iter() {
        let dx = ev.delta.x / screen.height; // No typo, height is reference
        let dy = ev.delta.y / screen.height;
        if lmb.pressed(MouseButton::Left) {
            for handle in query.iter_mut() {
                let fractal = mats.get_mut(handle).unwrap();
                fractal.center.x -= dx / fractal.scale;
                fractal.center.y -= dy / fractal.scale;
            }
        }
    }
}

fn fractal_start(
    mut mouse_event: EventReader<MouseMotion>,
    mut query: Query<&Handle<MandelbrotMaterial>>,
    screen: Res<Screen>,
    rmb: Res<Input<MouseButton>>,
    mut mats: ResMut<Assets<MandelbrotMaterial>>,
) {
    if rmb.pressed(MouseButton::Right) {
        for ev in mouse_event.iter() {
            for handle in query.iter_mut() {
                let fractal = mats.get_mut(handle).unwrap();
                let dx = ev.delta.x / screen.height;
                let dy = ev.delta.y / screen.height;
                fractal.start.x -= dx / 4.;
                fractal.start.y -= dy / 4.;
                println!("start = {}, {}", fractal.start.x, fractal.start.y);
            }
        }
    }
}

fn fractal_zoom(
    mut query: Query<&Handle<MandelbrotMaterial>>,
    mut scroll_event: EventReader<MouseWheel>,
    mut mats: ResMut<Assets<MandelbrotMaterial>>,
    mouse_pos: Res<MousePos>,
) {
    for ev in scroll_event.iter() {
        let amt = ev.y * 0.05;
        let factor = 1.0 + amt;
        for handle in query.iter_mut() {
            let fractal = mats.get_mut(handle).unwrap();

            // Correct center position to zoom towards mouse position
            fractal.center.x += mouse_pos.x / fractal.scale * amt / 2.0;
            fractal.center.y -= mouse_pos.y / fractal.scale * amt / 2.0;
            fractal.scale *= factor;
        }
    }
}

fn cursor_moved(
    mut cursor_moved: EventReader<CursorMoved>,
    mut mouse_pos: ResMut<MousePos>,
    screen: Res<Screen>,
) {
    for vm in cursor_moved.iter() {
        mouse_pos.x = ((vm.position.x / screen.width) - 0.5) * 2.0 * screen.aspect;
        mouse_pos.y = ((vm.position.y / screen.height) - 0.5) * 2.0;
    }
}

fn change_iters(
    mut scroll_event: EventReader<MouseWheel>,
    mut mats: ResMut<Assets<MandelbrotMaterial>>,
    mut query: Query<&Handle<MandelbrotMaterial>>,
) {
    for ev in scroll_event.iter() {
        if (ev.x.abs() < 0.05) {
            continue;
        }

        let dir = ev.x.signum() as i32;
        for handle in query.iter_mut() {
            let fractal = mats.get_mut(handle).unwrap();

            fractal.iters += dir;
            fractal.iters = fractal.iters.max(2);
            println!("Iterations: {}", fractal.iters);
        }
    }
}

fn reset_start(
    mut cl: EventReader<DoubleclickEvent>,
    mut query: Query<&Handle<MandelbrotMaterial>>,
    mut mats: ResMut<Assets<MandelbrotMaterial>>,
) {
    for ev in cl.iter() {
        if ev.button == MouseButton::Right {
            for handle in query.iter_mut() {
                let fractal = mats.get_mut(handle).unwrap();
                fractal.start.x = 0.0;
                fractal.start.y = 0.0;
            }
        }
    }
}

fn reset_view(
    mut cl: EventReader<DoubleclickEvent>,
    mut query: Query<&Handle<MandelbrotMaterial>>,
    mut mats: ResMut<Assets<MandelbrotMaterial>>,
) {
    for ev in cl.iter() {
        if ev.button == MouseButton::Left {
            for handle in query.iter_mut() {
                let fractal = mats.get_mut(handle).unwrap();
                fractal.center.x = -0.4;
                fractal.center.y = 0.0;
                fractal.scale = 0.4;
                fractal.iters = 64;
            }
        }
    }
}
