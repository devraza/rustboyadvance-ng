/// Struct containing everything
///
use std::cell::RefCell;
use std::rc::Rc;

use super::arm7tdmi::{Core, DecodedInstruction};
use super::cartridge::Cartridge;
use super::gpu::*;
use super::interrupt::*;
use super::ioregs::IoRegs;
use super::sysbus::SysBus;
use super::EmuIoDev;
use super::GBAResult;
use crate::backend::*;

#[derive(Debug)]
pub struct IoDevices {
    pub intc: InterruptController,
    pub gpu: Gpu,
}

impl IoDevices {
    pub fn new() -> IoDevices {
        IoDevices {
            intc: InterruptController::new(),
            gpu: Gpu::new(),
        }
    }
}

pub struct GameBoyAdvance {
    backend: Box<EmulatorBackend>,
    pub cpu: Core,
    pub sysbus: SysBus,

    pub io: Rc<RefCell<IoDevices>>,
}

impl GameBoyAdvance {
    pub fn new(
        cpu: Core,
        bios_rom: Vec<u8>,
        gamepak: Cartridge,
        backend: Box<EmulatorBackend>,
    ) -> GameBoyAdvance {
        let io = Rc::new(RefCell::new(IoDevices::new()));

        let ioregs = IoRegs::new(io.clone());
        let sysbus = SysBus::new(bios_rom, gamepak, ioregs);

        GameBoyAdvance {
            backend: backend,
            cpu: cpu,
            sysbus: sysbus,

            io: io.clone(),
        }
    }

    pub fn frame(&mut self) {
        self.update_key_state();
        while self.io.borrow().gpu.state != GpuState::VBlank {
            let cycles = self.emulate_cpu();
            self.emulate_peripherals(cycles);
        }
        self.backend.render(self.io.borrow().gpu.render());
        while self.io.borrow().gpu.state == GpuState::VBlank {
            let cycles = self.emulate_cpu();
            self.emulate_peripherals(cycles);
        }
    }

    fn update_key_state(&mut self) {
        self.sysbus.ioregs.keyinput = self.backend.get_key_state();
    }

    pub fn emulate_cpu(&mut self) -> usize {
        let previous_cycles = self.cpu.cycles;
        self.cpu.step(&mut self.sysbus).unwrap();
        self.cpu.cycles - previous_cycles
    }

    pub fn emulate_peripherals(&mut self, cycles: usize) {
        let mut io = self.io.borrow_mut();
        let (_, irq) = io.gpu.step(cycles, &mut self.sysbus);
        if let Some(irq) = irq {
            io.intc.request_irq(&mut self.cpu, irq);
        }
    }

    pub fn step(&mut self) -> GBAResult<DecodedInstruction> {
        let previous_cycles = self.cpu.cycles;
        let executed_insn = self.cpu.step_one(&mut self.sysbus)?;
        let cycles = self.cpu.cycles - previous_cycles;

        self.emulate_peripherals(cycles);

        if self.io.borrow().gpu.state == GpuState::HBlank {
            self.backend.render(self.io.borrow().gpu.render());
        }

        Ok(executed_insn)
    }
}
