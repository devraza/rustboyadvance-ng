# rustboyadvance-ng

Nintendo GameBoy Advance™ emulator and debugger, written in Rust.

> This repository is a fork of [rustboyadvance-ng](https://github.com/michelhe/rustboyadvance-ng), with minimal non-feature changes.

# Project structure
* `core/` - Main emulator crate that ties everything together 
* `arm7tdmi/` - Emulation of the Arm7tdmi processor
* `app/` -  Contains the desktop application built with `sdl2`

## External content
The file at [`external/gamecontrollerdb.txt`](./external/gamecontrollerdb.txt) is not my work - it is sourced from [this GitHub repository](https://github.com/mdqinc/SDL_GameControllerDB) and covered by the appropriate license present in said repository.

## Usage
You will need to specify the BIOS file to run with, using the `--bios` command-line argument. Note that there is such a BIOS ROM present in this directory at [`./external/gba_bios.bin`](./external/gba_bios.bin).

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
