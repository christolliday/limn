## Limn GUI library.

[![Build Status](https://travis-ci.org/christolliday/limn.svg?branch=master)](https://travis-ci.org/christolliday/limn)
[![Build status](https://ci.appveyor.com/api/projects/status/jheej7tmkntqa8d4/branch/master?svg=true)](https://ci.appveyor.com/project/christolliday/limn/branch/master)

An early stage, cross platform GUI library in Rust aiming for high performance, a composable widget system, and a small core API.

**WARNING: Has only been tested on X11, serious bugs exist and all APIs are likely to change.**

![screenshot](assets/screenshot.png)

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.

### Running examples under NixOS

winit needs X11 libraries at runtime. To get them on NixOS, you can create a `default.nix` file with the following content:

```
with import <nixpkgs> {}; {
  cargoEnv = stdenv.mkDerivation {
    name = "limn";
    shellHook = with xorg; ''
      export LD_LIBRARY_PATH=/run/opengl-driver/lib/:${lib.makeLibraryPath (with xorg; [libX11 libXcursor libXxf86vm libXi libXrandr xinput zlib])}
    '';
  };
}
```

then, running example should work as

```
$ nix-shell --run bash
$ cargo run --release --example crud
```
