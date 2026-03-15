# Jumper

> A fast little endless runner built with **Rust** and **raylib**.

Jumper is an arcade-style score chaser with double jumps, collectible coins, escalating speed, and fully procedural audio. It is small, responsive, and easy to run locally.

## Showcase

<p align="center">
	<img src="recording.webp" alt="Animated gameplay showcase for Jumper" width="900" />
</p>

## Highlights

- **Endless runner gameplay** with quick restart loops
- **Double jump movement** for tighter obstacle control
- **Multiple obstacle types** to keep runs unpredictable
- **Collectible coins** that boost your score
- **Difficulty ramp** with increasing world speed
- **Best score tracking** for repeat runs
- **Procedural sound effects** generated in code
- **Procedural background music** with in-game mute toggles
- **Animated scenery and particles** for extra polish

## Controls

| Key | Action |
| --- | --- |
| `Space` / `Up` / `W` | Jump / double jump |
| `R` | Restart the current run |
| `M` | Mute or unmute sound effects |
| `B` | Mute or unmute background music |
| `Esc` | Quit |

## Running the game

### Requirements

- Rust toolchain
- A desktop environment supported by `raylib`

### Start in development

```bash
cargo run
```

### Run an optimized build

```bash
cargo run --release
```

## Built with

- [Rust](https://www.rust-lang.org/)
- [raylib-rs](https://github.com/raylib-rs/raylib-rs)
- [rand](https://crates.io/crates/rand)

## License

This project is available under the terms of the [LICENSE](LICENSE).
