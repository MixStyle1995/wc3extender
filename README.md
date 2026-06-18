# WC3Extender

WIP Warcraft III feature extender. Aiming to be roughly the SKSE equivalent for Warcraft III.

## Status

**Done**
- Native registration
- Mounting MPQ files
- Frames API (halfway working)
- Loading lua scripts from maps
- Barebones plugins api

**Todo**
- Support 1.29 map data
- Polish and refactor existing code

## Layout

- `wc3sys/` — injected DLL / runtime
- `wc3/` — helper + plugin ABI crate
- `wc3launcher` - The launcher

## Credits

- [W3CE](https://github.com/Warcraft-III-Community-Edition/W3CE/) for their implementation of frames.
