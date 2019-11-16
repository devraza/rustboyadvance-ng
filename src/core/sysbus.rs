use std::fmt;
use std::ops::Add;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use super::arm7tdmi::bus::Bus;
use super::arm7tdmi::Addr;
use super::cartridge::Cartridge;
use super::gpu::GpuState;
use super::iodev::IoDevices;

const VIDEO_RAM_SIZE: usize = 128 * 1024;
const WORK_RAM_SIZE: usize = 256 * 1024;
const INTERNAL_RAM_SIZE: usize = 32 * 1024;
const PALETTE_RAM_SIZE: usize = 1 * 1024;
const OAM_SIZE: usize = 1 * 1024;

pub const BIOS_ADDR: u32 = 0x0000_0000;
pub const EWRAM_ADDR: u32 = 0x0200_0000;
pub const IWRAM_ADDR: u32 = 0x0300_0000;
pub const IOMEM_ADDR: u32 = 0x0400_0000;
pub const PALRAM_ADDR: u32 = 0x0500_0000;
pub const VRAM_ADDR: u32 = 0x0600_0000;
pub const OAM_ADDR: u32 = 0x0700_0000;
pub const GAMEPAK_WS0_ADDR: u32 = 0x0800_0000;
pub const GAMEPAK_MIRROR_WS0_ADDR: u32 = 0x0900_0000;
pub const GAMEPAK_WS1_ADDR: u32 = 0x0A00_0000;
pub const GAMEPAK_WS2_ADDR: u32 = 0x0C00_0000;

#[derive(Debug, Copy, Clone)]
pub enum MemoryAccessType {
    NonSeq,
    Seq,
}

impl fmt::Display for MemoryAccessType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MemoryAccessType::NonSeq => "N",
                MemoryAccessType::Seq => "S",
            }
        )
    }
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MemoryAccessWidth {
    MemoryAccess8,
    MemoryAccess16,
    MemoryAccess32,
}

impl Add<MemoryAccessWidth> for MemoryAccessType {
    type Output = MemoryAccess;

    fn add(self, other: MemoryAccessWidth) -> Self::Output {
        MemoryAccess(self, other)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MemoryAccess(pub MemoryAccessType, pub MemoryAccessWidth);

impl fmt::Display for MemoryAccess {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}-Cycle ({:?})", self.0, self.1)
    }
}

#[derive(Debug)]
pub struct BoxedMemory {
    pub mem: Box<[u8]>,
}

impl BoxedMemory {
    pub fn new(boxed_slice: Box<[u8]>) -> BoxedMemory {
        BoxedMemory { mem: boxed_slice }
    }
}

impl Bus for BoxedMemory {
    fn read_32(&self, addr: Addr) -> u32 {
        (&self.mem[addr as usize..])
            .read_u32::<LittleEndian>()
            .unwrap()
    }

    fn read_16(&self, addr: Addr) -> u16 {
        (&self.mem[addr as usize..])
            .read_u16::<LittleEndian>()
            .unwrap()
    }

    fn read_8(&self, addr: Addr) -> u8 {
        (&self.mem[addr as usize..])[0]
    }

    fn write_32(&mut self, addr: Addr, value: u32) {
        (&mut self.mem[addr as usize..])
            .write_u32::<LittleEndian>(value)
            .unwrap()
    }

    fn write_16(&mut self, addr: Addr, value: u16) {
        (&mut self.mem[addr as usize..])
            .write_u16::<LittleEndian>(value)
            .unwrap()
    }

    fn write_8(&mut self, addr: Addr, value: u8) {
        (&mut self.mem[addr as usize..]).write_u8(value).unwrap()
    }
}

#[derive(Debug)]
struct DummyBus([u8; 4]);

impl Bus for DummyBus {
    fn read_32(&self, _addr: Addr) -> u32 {
        0
    }

    fn read_16(&self, _addr: Addr) -> u16 {
        0
    }

    fn read_8(&self, _addr: Addr) -> u8 {
        0
    }

    fn write_32(&mut self, _addr: Addr, _value: u32) {}

    fn write_16(&mut self, _addr: Addr, _value: u16) {}

    fn write_8(&mut self, _addr: Addr, _value: u8) {}
}

#[derive(Debug)]
pub struct SysBus {
    pub io: IoDevices,

    bios: BoxedMemory,
    onboard_work_ram: BoxedMemory,
    internal_work_ram: BoxedMemory,
    pub palette_ram: BoxedMemory,
    pub vram: BoxedMemory,
    pub oam: BoxedMemory,
    gamepak: Cartridge,
    dummy: DummyBus,

    pub trace_access: bool,
}

impl SysBus {
    pub fn new(io: IoDevices, bios_rom: Vec<u8>, gamepak: Cartridge) -> SysBus {
        SysBus {
            io: io,

            bios: BoxedMemory::new(bios_rom.into_boxed_slice()),
            onboard_work_ram: BoxedMemory::new(vec![0; WORK_RAM_SIZE].into_boxed_slice()),
            internal_work_ram: BoxedMemory::new(vec![0; INTERNAL_RAM_SIZE].into_boxed_slice()),
            palette_ram: BoxedMemory::new(vec![0; PALETTE_RAM_SIZE].into_boxed_slice()),
            vram: BoxedMemory::new(vec![0; VIDEO_RAM_SIZE].into_boxed_slice()),
            oam: BoxedMemory::new(vec![0; OAM_SIZE].into_boxed_slice()),
            gamepak: gamepak,
            dummy: DummyBus([0; 4]),

            trace_access: false,
        }
    }

    fn map(&self, addr: Addr) -> (&dyn Bus, Addr) {
        let ofs = addr & 0x00ff_ffff;
        match addr & 0xff000000 {
            BIOS_ADDR => {
                if ofs >= 0x4000 {
                    (&self.dummy, ofs) // TODO return last fetched opcode
                } else {
                    (&self.bios, ofs)
                }
            }
            EWRAM_ADDR => (&self.onboard_work_ram, ofs & 0x3_ffff),
            IWRAM_ADDR => (&self.internal_work_ram, ofs & 0x7fff),
            IOMEM_ADDR => (&self.io, {
                if ofs & 0xffff == 0x8000 {
                    0x800
                } else {
                    ofs & 0x7ff
                }
            }),
            PALRAM_ADDR => (&self.palette_ram, ofs & 0x3ff),
            VRAM_ADDR => (&self.vram, {
                let mut ofs = ofs & ((VIDEO_RAM_SIZE as u32) - 1);
                if ofs > 0x18000 {
                    ofs -= 0x8000;
                }
                ofs
            }),
            OAM_ADDR => (&self.oam, ofs & 0x3ff),
            GAMEPAK_WS0_ADDR | GAMEPAK_MIRROR_WS0_ADDR | GAMEPAK_WS1_ADDR | GAMEPAK_WS2_ADDR => {
                (&self.gamepak, addr & 0x01ff_ffff)
            }
            _ => (&self.dummy, ofs),
        }
    }

    /// TODO proc-macro for generating this function
    fn map_mut(&mut self, addr: Addr) -> (&mut dyn Bus, Addr) {
        let ofs = addr & 0x00ff_ffff;
        match addr & 0xff000000 {
            BIOS_ADDR => (&mut self.dummy, ofs),
            EWRAM_ADDR => (&mut self.onboard_work_ram, ofs & 0x3_ffff),
            IWRAM_ADDR => (&mut self.internal_work_ram, ofs & 0x7fff),
            IOMEM_ADDR => (&mut self.io, {
                if ofs & 0xffff == 0x8000 {
                    0x800
                } else {
                    ofs & 0x7ff
                }
            }),
            PALRAM_ADDR => (&mut self.palette_ram, ofs & 0x3ff),
            VRAM_ADDR => (&mut self.vram, {
                let mut ofs = ofs & ((VIDEO_RAM_SIZE as u32) - 1);
                if ofs > 0x18000 {
                    ofs -= 0x8000;
                }
                ofs
            }),
            OAM_ADDR => (&mut self.oam, ofs & 0x3ff),
            GAMEPAK_WS0_ADDR | GAMEPAK_MIRROR_WS0_ADDR | GAMEPAK_WS1_ADDR | GAMEPAK_WS2_ADDR => {
                (&mut self.gamepak, addr & 0x01ff_ffff)
            }
            _ => (&mut self.dummy, ofs),
        }
    }

    pub fn get_cycles(&self, addr: Addr, access: MemoryAccess) -> usize {
        let nonseq_cycles = [4, 3, 2, 8];
        let seq_cycles = [2, 1];

        let mut cycles = 0;

        // TODO handle EWRAM accesses
        match addr & 0xff000000 {
            EWRAM_ADDR => match access.1 {
                MemoryAccessWidth::MemoryAccess32 => cycles += 6,
                _ => cycles += 3,
            },
            OAM_ADDR | VRAM_ADDR | PALRAM_ADDR => {
                match access.1 {
                    MemoryAccessWidth::MemoryAccess32 => cycles += 2,
                    _ => cycles += 1,
                }
                if self.io.gpu.state == GpuState::HDraw {
                    cycles += 1;
                }
            }
            GAMEPAK_WS0_ADDR | GAMEPAK_MIRROR_WS0_ADDR => match access.0 {
                MemoryAccessType::NonSeq => match access.1 {
                    MemoryAccessWidth::MemoryAccess32 => {
                        cycles += nonseq_cycles[self.io.waitcnt.ws0_first_access() as usize];
                        cycles += seq_cycles[self.io.waitcnt.ws0_second_access() as usize];
                    }
                    _ => {
                        cycles += nonseq_cycles[self.io.waitcnt.ws0_first_access() as usize];
                    }
                },
                MemoryAccessType::Seq => {
                    cycles += seq_cycles[self.io.waitcnt.ws0_second_access() as usize];
                    if access.1 == MemoryAccessWidth::MemoryAccess32 {
                        cycles += seq_cycles[self.io.waitcnt.ws0_second_access() as usize];
                    }
                }
            },
            GAMEPAK_WS1_ADDR | GAMEPAK_WS2_ADDR => {
                panic!("unimplemented - need to refactor code with a nice macro :(")
            }
            _ => {}
        }

        cycles
    }
}

impl Bus for SysBus {
    fn read_32(&self, addr: Addr) -> u32 {
        let (dev, addr) = self.map(addr);
        dev.read_32(addr & 0x1ff_fffc)
    }

    fn read_16(&self, addr: Addr) -> u16 {
        if self.trace_access {
            println!("[TRACE] read_32 addr={:x}", addr);
        }
        let (dev, addr) = self.map(addr);
        dev.read_16(addr & 0x1ff_fffe)
    }

    fn read_8(&self, addr: Addr) -> u8 {
        if self.trace_access {
            println!("[TRACE] read_32 addr={:x}", addr);
        }
        let (dev, addr) = self.map(addr);
        dev.read_8(addr & 0x1ff_ffff)
    }

    fn write_32(&mut self, addr: Addr, value: u32) {
        let (dev, addr) = self.map_mut(addr);
        dev.write_32(addr & 0x1ff_fffc, value);
    }

    fn write_16(&mut self, addr: Addr, value: u16) {
        let (dev, addr) = self.map_mut(addr);
        dev.write_16(addr & 0x1ff_fffe, value);
    }

    fn write_8(&mut self, addr: Addr, value: u8) {
        let (dev, addr) = self.map_mut(addr);
        dev.write_8(addr & 0x1ff_ffff, value);
    }
}
