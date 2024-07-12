# Qevy
## A plugin that adds Quake .map file support for the Bevy game engine

Supported Bevy Versions: 0.13
Supported Physics Engines: XPBD, Rapier

# Features
- [x] Geometry ✔️
- [x] Only build geometry that has textures ✔️
- [x] A post build map hook for spawning your entities ✔️
- [x] XPBD physics ✔️
- [x] Rapier physics ✔️
- [ ] Phong normals ❌ coming soon ❌
- [x] Triggers ✔️

## Example project & TrenchBroom

Run the example project with `cargo run --release --features="xpbd" --example first_person`

Under the example folder, you will find an example of how to use this plugin.

1. The `Quevy Example` folder inside the `trenchbroom` folder is meant to be moved into the TrenchBroom game's folder (example: `C:\Users\YOUR USER\AppData\Roaming\TrenchBroom\games\Qevy Example`)
2. Open TrenchBroom and select the `Qevy Example` game.
3. Set the game's path to the `example/assets` folder.
4. Open the `example.map` file located in `example/assets`.

## Automatic Config File Generation

Qevy supports the automatic generation of a configuration file on startup.
Qevy uses Bevy's Reflection to generate the file. All you have to do is to add the `AutoCreateConfigPlugin`:

```rust
App::new()
    .add_plugins((
        DefaultPlugins,
        qevy::MapAssetLoaderPlugin::default(),
        qevy::auto_create_config::AutoCreateConfigPlugin::new("qevy_example.fgd".into()),
    ));
```

You can set the path and name of the file to be generated. It will be stored in bevy's `assets` folder.

Now Qevy will try to generate a configuration file on startup, based on all registered structs.
To register a struct, all you have to do is just:

```rust
App::new()
    .register_qevy_entity::<TestBaseClass>();
```

And a qevy entity is a normal rust struct, which simply derives `Reflect` and `QevyEntity`:

```rust
#[derive(Reflect, Default, QevyEntity)]
#[reflect(QevyEntity, Default)]
#[qevy_entity(entity_type = "Base")]
struct TestBaseClass;
```

You can find an example of how to use this in the `example` folder, called `exporting_config.rs`.

## Special Thanks
Special thanks to Shfty over at Qodot for the wonderful [Shambler crate](https://github.com/QodotPlugin/shambler) which handles much of the heavy lifting.

## License

This project is licensed under both MIT and Apache 2.0. You may use this project under the terms of either license.
