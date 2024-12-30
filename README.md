日本語復習しよう！
---
A port of [日本語を勉強しましょう！](https://github.com/itscrystalline/nihongowobenkyoushimashou)
Rewritten in Rust for **Blazingly Fast🚀🚀🚀** speeds!

It's not the best Rust code, probably also not the fastest, but it is *decently* fast.

for comparison, pulling the same 20 cards from the database took ~140ms in python, and 1-5ms in rust.

not shocking, but it is something, anywho

### binaries

---
this repo can compile 2 binaries. one is the main quiz program, nihongofukushuushiyou. it can be compiled with

```text
cargo build --release
```

the other is an importer program, nyuuryokusha (入力者). you can compile it with

```text
cargo build --release --bin nyuuryokusha
```

to configure the log level, provide the binary with `RUST_LOG=(your log level)`.
for example, to show all debug logs, run in the terminal:

```text
RUST_LOG=debug /path/to/nihongofukushuushiyou
```

thats it. have fun with this silly thing i made i guess :3