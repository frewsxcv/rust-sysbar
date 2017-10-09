# rust-sysbar

Library for interacting with the system's taskbar / tray / statusbar. It aims to be cross-platform, but currently only supports macOS. If have some extra time and are interested in implementing this for other platforms, contributions would be greatly appreciated! This project is a fork of [rs-barfly](https://github.com/jmquigs/rs-barfly).

## Example

```rust
let mut bar = sysbar::Sysbar::new("Foo");

bar.add_item(
    "Say 'bar'",
    Box::new(move || {
        println!("bar");
    }),
);

bar.add_quit_item("Quit");

bar.display();
```

![Resulting screenshot of code above](http://i.imgur.com/mEI6Mxy.png)

## See also

* [systray-rs](https://github.com/qdot/systray-rs)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
