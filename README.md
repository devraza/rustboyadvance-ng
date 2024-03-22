# rustboyadvance-ng

Nintendo GameBoy Advance™ emulator and debugger, written in Rust.

> This repository is a fork of [rustboyadvance-ng](https://github.com/michelhe/rustboyadvance-ng), with minimal non-feature changes.

# Project structure
* `core/` - Main emulator crate that ties everything together 
* `arm7tdmi/` - Emulation of the Arm7tdmi processor
* `platform/` - Constains executables & application built with `rustboyadvance-core`
    * `platform/rustbodyadvance-wasm` - Web emulator powered by WebAssembly
    * `platform/rustbodyadvance-sdl2` - Desktop application built with sdl2
    * `platform/rustbodyadvance-minifb` - Desktop application built with minifb, *not maintained*.
    * `platform/rustbodyadvance-jni` - Java JNI binidngs for the emulator.
    * `platform/android` - A PoC Android application.

## External content
The file at [`external/gamecontrollerdb.txt`](./external/gamecontrollerdb.txt) is not my work - it is sourced from [this GitHub repository](https://github.com/mdqinc/SDL_GameControllerDB) and covered by the appropriate license present in said repository.

## Key bindings
GBA key bindings:

| Keyboard  	| GBA      	|
|-----------	|----------	|
| Up        	| Up       	|
| Down      	| Down     	|
| Left      	| Right    	|
| Right     	| Right    	|
| Z         	| B Button 	|
| X         	| A Button 	|
| Return    	| Start    	|
| Backspace 	| Select   	|
| A         	| L        	|
| S         	| R        	|

Special key bindings
| Key          	| Function          	|
|--------------	|--------------------	|
| Space (hold) 	| Disable 60fps cap  	|
| F1		| Custom debugger (requires --features debugger) |
| F2		| Spawn gdbserver (experimetnal, requires --features gdb) |
| F5           	| Save snapshot file 	|
| F9           	| Load snapshot file 	|
