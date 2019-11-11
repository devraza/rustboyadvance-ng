use std::ffi::OsStr;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use std::time;

#[macro_use]
extern crate clap;

extern crate zip;

use clap::{App, ArgMatches};

extern crate rustboyadvance_ng;

use rustboyadvance_ng::backend::*;
use rustboyadvance_ng::core::arm7tdmi::Core;
use rustboyadvance_ng::core::cartridge::Cartridge;
use rustboyadvance_ng::core::{GBAError, GBAResult, GameBoyAdvance};
use rustboyadvance_ng::debugger::Debugger;
use rustboyadvance_ng::util::read_bin_file;

fn load_rom(path: &str) -> GBAResult<Vec<u8>> {
    if path.ends_with(".zip") {
        let zipfile = File::open(path)?;
        let mut archive = zip::ZipArchive::new(zipfile)?;
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            if file.name().ends_with(".gba") {
                let mut buf = Vec::new();
                file.read_to_end(&mut buf)?;
                return Ok(buf);
            }
        }
        panic!("no .gba file contained in the zip file");
    } else {
        let buf = read_bin_file(path)?;
        Ok(buf)
    }
}

fn run_emulator(matches: &ArgMatches) -> GBAResult<()> {
    let skip_bios = matches.occurrences_of("skip_bios") != 0;
    let no_framerate_limit = matches.occurrences_of("no_framerate_limit") != 0;
    let debug = matches.occurrences_of("debug") != 0;

    let backend: Box<EmulatorBackend> = match matches.value_of("backend") {
        Some("sdl2") => Box::new(Sdl2Backend::new()),
        Some("minifb") => Box::new(MinifbBackend::new()),
        // None => DummyBackend::new(),
        None => Box::new(DummyBackend::new()),
        _ => unreachable!(),
    };

    let bios_bin = read_bin_file(matches.value_of("bios").unwrap_or_default())?;

    let rom_bin = load_rom(matches.value_of("game_rom").unwrap())?;
    let cart = Cartridge::new(rom_bin);
    println!("loaded rom: {:#?}", cart.header);

    let mut core = Core::new();
    if skip_bios {
        core.skip_bios();
    }

    let mut gba = GameBoyAdvance::new(core, bios_bin, cart, backend);

    if debug {
        gba.cpu.set_verbose(true);
        let mut debugger = Debugger::new(gba);
        println!("starting debugger...");
        debugger.repl()?;
        println!("ending debugger...");
    } else {
        let frame_time = time::Duration::new(0, 1_000_000_000u32 / 60);
        loop {
            let start_time = time::Instant::now();
            gba.frame();
            if !no_framerate_limit {
                let time_passed = start_time.elapsed();
                let delay = frame_time.checked_sub(time_passed);
                match delay {
                    None => {}
                    Some(delay) => {
                        ::std::thread::sleep(delay);
                    }
                };
            }
        }
    }

    Ok(())
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let result = run_emulator(&matches);

    if let Err(err) = result {
        println!("Got an error: {:?}", err);
    }
}
