use crate::arm7tdmi::bus::Bus;
use crate::arm7tdmi::{Addr, CpuState};
use crate::disass::Disassembler;
use crate::GBAError;

use super::{parser::Value, Debugger, DebuggerError, DebuggerResult};
use super::palette_view::create_palette_view;

use ansi_term::Colour;

use colored::*;
use hexdump;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DisassMode {
    ModeArm,
    ModeThumb,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Command {
    Info,
    Step(usize),
    Continue,
    HexDump(Addr, usize),
    Disass(DisassMode, Addr, usize),
    AddBreakpoint(Addr),
    DelBreakpoint(Addr),
    PaletteView,
    ClearBreakpoints,
    ListBreakpoints,
    Reset,
    Quit,
}

impl Command {
    pub fn run(&self, debugger: &mut Debugger) {
        use Command::*;
        match *self {
            Info => println!("{}", debugger.gba.cpu),
            Step(count) => {
                for _ in 0..count {
                    if let Some(bp) = debugger.check_breakpoint() {
                        println!("hit breakpoint #0x{:08x}!", bp);
                        debugger.delete_breakpoint(bp);
                    } else {
                        match debugger.gba.step() {
                            Ok(insn) => {
                                print!(
                                    "{}\t{}",
                                    Colour::Black
                                        .bold()
                                        .italic()
                                        .on(Colour::White)
                                        .paint(format!("Executed at @0x{:08x}:", insn.get_pc(),)),
                                    insn
                                );
                                println!(
                                    "{}",
                                    Colour::Purple.dimmed().italic().paint(format!(
                                        "\t\t/// Next instruction at @0x{:08x}",
                                        debugger.gba.cpu.get_next_pc()
                                    ))
                                )
                            }
                            Err(GBAError::CpuError(e)) => {
                                println!("{}: {}", "cpu encountered an error".red(), e);
                                println!("cpu: {:x?}", debugger.gba.cpu)
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                println!("{}\n", debugger.gba.cpu);
            }
            Continue => loop {
                if let Some(bp) = debugger.check_breakpoint() {
                    println!("hit breakpoint #0x{:08x}!", bp);
                    debugger.delete_breakpoint(bp);
                    break;
                }
                match debugger.gba.step() {
                    Ok(insn) => {
                        println!(
                            "@0x{:08x}:\t{}",
                            insn.get_pc(),
                            Colour::Yellow.italic().paint(format!("{} ", insn))
                        );
                    }
                    Err(GBAError::CpuError(e)) => {
                        println!("{}: {}", "cpu encountered an error".red(), e);
                        println!("cpu: {:x?}", debugger.gba.cpu);
                        break;
                    }
                    _ => unreachable!(),
                };
            },
            HexDump(addr, nbytes) => {
                let bytes = debugger.gba.sysbus.get_bytes(addr);
                hexdump::hexdump(&bytes[0..nbytes]);
            }
            Disass(mode, addr, n) => {
                use crate::arm7tdmi::arm::ArmInstruction;
                use crate::arm7tdmi::thumb::ThumbInstruction;

                let bytes = debugger.gba.sysbus.get_bytes(addr);
                match mode {
                    DisassMode::ModeArm => {
                        let disass = Disassembler::<ArmInstruction>::new(addr, bytes);
                        for (_, line) in disass.take(n) {
                            println!("{}", line)
                        }
                    }
                    DisassMode::ModeThumb => {
                        let disass = Disassembler::<ThumbInstruction>::new(addr, bytes);
                        for (_, line) in disass.take(n) {
                            println!("{}", line)
                        }
                    }
                };
            }
            Quit => {
                print!("Quitting!");
                debugger.stop();
            }
            AddBreakpoint(addr) => {
                if !debugger.breakpoints.contains(&addr) {
                    let new_index = debugger.breakpoints.len();
                    debugger.breakpoints.push(addr);
                    println!("added breakpoint [{}] 0x{:08x}", new_index, addr);
                } else {
                    println!("breakpoint already exists!")
                }
            }
            DelBreakpoint(addr) => debugger.delete_breakpoint(addr),
            ClearBreakpoints => debugger.breakpoints.clear(),
            ListBreakpoints => {
                println!("breakpoint list:");
                for (i, b) in debugger.breakpoints.iter().enumerate() {
                    println!("[{}] 0x{:08x}", i, b)
                }
            }
            PaletteView => create_palette_view(debugger.gba.sysbus.get_bytes(0x0500_0000)),
            Reset => {
                println!("resetting cpu...");
                debugger.gba.cpu.reset();
                println!("cpu is restarted!")
            }
        }
    }
}

impl Debugger {
    fn get_disassembler_args(&self, args: Vec<Value>) -> DebuggerResult<(Addr, usize)> {
        match args.len() {
            2 => {
                let addr = self.val_address(&args[0])?;
                let n = self.val_number(&args[1])?;

                Ok((addr, n as usize))
            }
            1 => {
                let addr = self.val_address(&args[0])?;

                Ok((addr, 10))
            }
            0 => {
                if let Some(Command::Disass(_mode, addr, n)) = &self.previous_command {
                    Ok((*addr + (4 * (*n as u32)), 10))
                } else {
                    Ok((self.gba.cpu.get_next_pc(), 10))
                }
            }
            _ => {
                return Err(DebuggerError::InvalidCommandFormat(
                    "disass [addr] [n]".to_string(),
                ))
            }
        }
    }

    pub fn eval_command(&self, command: Value, args: Vec<Value>) -> DebuggerResult<Command> {
        let command = match command {
            Value::Identifier(command) => command,
            _ => {
                return Err(DebuggerError::InvalidCommand("expected a name".to_string()));
            }
        };

        match command.as_ref() {
            "i" | "info" => Ok(Command::Info),
            "s" | "step" => {
                let count = match args.len() {
                    0 => 1,
                    1 => self.val_number(&args[0])?,
                    _ => {
                        return Err(DebuggerError::InvalidCommandFormat(
                            "step [count]".to_string(),
                        ))
                    }
                };
                Ok(Command::Step(count as usize))
            }
            "c" | "continue" => Ok(Command::Continue),
            "x" | "hexdump" => {
                let (addr, n) = match args.len() {
                    2 => {
                        let addr = self.val_address(&args[0])?;
                        let n = self.val_number(&args[1])?;

                        (addr, n as usize)
                    }
                    1 => {
                        let addr = self.val_address(&args[0])?;

                        (addr, 0x100)
                    }
                    0 => {
                        if let Some(Command::HexDump(addr, n)) = self.previous_command {
                            (addr + (4 * n as u32), 0x100)
                        } else {
                            (self.gba.cpu.get_reg(15), 0x100)
                        }
                    }
                    _ => {
                        return Err(DebuggerError::InvalidCommandFormat(
                            "xxd [addr] [n]".to_string(),
                        ))
                    }
                };
                Ok(Command::HexDump(addr, n))
            }
            "d" | "disass" => {
                let (addr, n) = self.get_disassembler_args(args)?;

                let m = match self.gba.cpu.cpsr.state() {
                    CpuState::ARM => DisassMode::ModeArm,
                    CpuState::THUMB => DisassMode::ModeThumb,
                };
                Ok(Command::Disass(m, addr, n))
            }
            "da" | "disass-arm" => {
                let (addr, n) = self.get_disassembler_args(args)?;

                Ok(Command::Disass(DisassMode::ModeArm, addr, n))
            }
            "dt" | "disass-thumb" => {
                let (addr, n) = self.get_disassembler_args(args)?;

                Ok(Command::Disass(DisassMode::ModeThumb, addr, n))
            }
            "b" | "break" => {
                if args.len() != 1 {
                    Err(DebuggerError::InvalidCommandFormat(
                        "break <addr>".to_string(),
                    ))
                } else {
                    let addr = self.val_address(&args[0])?;
                    Ok(Command::AddBreakpoint(addr))
                }
            }
            "bd" | "breakdel" => match args.len() {
                0 => Ok(Command::ClearBreakpoints),
                1 => {
                    let addr = self.val_address(&args[0])?;
                    Ok(Command::DelBreakpoint(addr))
                }
                _ => Err(DebuggerError::InvalidCommandFormat(String::from(
                    "breakdel [addr]",
                ))),
            },
            "palette-view" => Ok(Command::PaletteView),
            "bl" => Ok(Command::ListBreakpoints),
            "q" | "quit" => Ok(Command::Quit),
            "r" | "reset" => Ok(Command::Reset),
            _ => Err(DebuggerError::InvalidCommand(command)),
        }
    }
}
