use std::io::{self, Write};

const MEMORY_SIZE: usize = 4096;
const NREGS: usize = 16;

const IP: usize = 0;

pub struct Machine {
    // Memory
    mem: [u8; MEMORY_SIZE],
    // Registers
    reg: [u32; NREGS]
}

#[derive(Debug)]
pub enum MachineError {
    // The program tried to access an invalid memory address.
    InvalidMemoryAccess,
    // The program tried to access an invalid register.
    InvalidRegisterAccess,
    // The program tried to execute an invalid instruction.
    InvalidInstruction,
    // The program failed to write to the output.
    IOError(io::Error)
}

impl From<io::Error> for MachineError {
    fn from(error: io::Error) -> Self {
        MachineError::IOError(error)
    }
}

impl Machine {
    /// Create a new machine in its reset state. The `memory` parameter will
    /// be copied at the beginning of the machine memory.
    ///
    /// # Panics
    /// This function panics when `memory` is larger than the machine memory.
    pub fn new(memory: &[u8]) -> Self {
        if memory.len()>MEMORY_SIZE {
            panic!("The memory length is bigger than expected. It must not be bigger than {}", MEMORY_SIZE)
        } else {
            let mut mem = [0; MEMORY_SIZE];
            mem[..memory.len()].copy_from_slice(memory);
            let reg = [0; NREGS];
            Machine {mem, reg}
        }
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on `fd`.
    pub fn run_on<T: Write>(&mut self, fd: &mut T) -> Result<(), MachineError> {
        while !self.step_on(fd)? {}
        Ok(())
    }

    /// Run until the program terminates or until an error happens.
    /// If output instructions are run, they print on standard output.
    pub fn run(&mut self) -> Result<(), MachineError> {
        self.run_on(&mut io::stdout().lock())
    }

    /// Execute the next instruction by doing the following steps:
    ///   - decode the instruction located at IP (register 0)
    ///   - increment the IP by the size of the instruction
    ///   - execute the decoded instruction
    ///
    /// If output instructions are run, they print on `fd`.
    /// If an error happens at either of those steps, an error is
    /// returned.
    ///
    /// In case of success, `true` is returned if the program is
    /// terminated (upon encountering an exit instruction), or
    /// `false` if the execution must continue.
    /// 
    pub fn step_on<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        /*step_on(): takes a Write-implementing descriptor (for the out and out number 
        instructions), and execute just one instruction */
        let adr = self.reg[IP];
        if adr as usize >= MEMORY_SIZE {
            return Err(MachineError::InvalidMemoryAccess);
        }
        let inst = self.mem[adr as usize];
        match inst {
            1 => self.move_if(),
            2 => self.store(),
            3 => self.load(),
            4 => self.loadimm(),
            5 => self.sub(),
            6 => self.out(fd),
            7 => self.exit(),
            8 => self.out_number(fd),
            _ => Err(MachineError::InvalidInstruction)
        }  
    }

    /// Similar to [step_on](Machine::step_on).
    /// If output instructions are run, they print on standard output.
    pub fn step(&mut self) -> Result<bool, MachineError> {
        self.step_on(&mut io::stdout().lock())
    }

    /// Reference onto the machine current set of registers.
    pub fn regs(&self) -> &[u32] {
        &self.reg
    }

    /// Sets a register to the given value.
    pub fn set_reg(&mut self, reg: usize, value: u32) -> Result<(), MachineError> {
        if reg >= NREGS {
            return Err(MachineError::InvalidRegisterAccess);
        }
        self.reg[reg] = value;
        Ok(())
    }

    /// Reference onto the machine current memory.
    pub fn memory(&self) -> &[u8] {
        &self.mem
    }

    /*move if
    1 reg_a reg_b reg_c: if register reg_c contains a non-zero value, copy the content of 
    register reg_b into register reg_a; otherwise do nothing. */
    fn move_if(&mut self) -> Result<bool, MachineError> {
        let adr = &mut self.reg[IP];
        if (*adr as usize + 3) >= MEMORY_SIZE {
            return Err(MachineError::InvalidMemoryAccess);
        }
        let reg_a = self.mem[*adr as usize + 1] as usize;
        let reg_b = self.mem[*adr as usize + 2] as usize;
        let reg_c = self.mem[*adr as usize + 3] as usize;
        if reg_a >= NREGS || reg_b >= NREGS || reg_c >= NREGS {
            return Err(MachineError::InvalidRegisterAccess);
        }
        /* I need to update adr here so that adr goes out of scope and can access other 
        registers directly */
        *adr += 4; 
        if self.reg[reg_c] != 0 {
            self.reg[reg_a] = self.reg[reg_b];
        }
        Ok(false)
    }
    /*store
    2 reg_a reg_b: store the content of register reg_b into the memory starting at address 
    pointed by register reg_a using little-endian representation. */
    fn store(&mut self) -> Result<bool, MachineError>{
        let adr = &mut self.reg[IP];
        if (*adr as usize + 2)  >= MEMORY_SIZE {
            return Err(MachineError::InvalidMemoryAccess);
        }
        let reg_a = self.mem[*adr as usize + 1] as usize;
        let reg_b = self.mem[*adr as usize + 2] as usize;
        if reg_a >= NREGS || reg_b >= NREGS {
            return Err(MachineError::InvalidRegisterAccess);
        }
        /* I need to update adr here so that adr goes out of scope and can access other 
        registers directly */
        *adr += 3; 
        if ((self.reg[reg_a] as usize)+3) >= MEMORY_SIZE {
            Err(MachineError::InvalidMemoryAccess)
        } else {
            let src = self.reg[reg_b].to_le_bytes();
            let dst = &mut self.mem[self.reg[reg_a] as usize..(self.reg[reg_a] + 4) as usize];
            dst.copy_from_slice(&src);
            Ok(false)
        }
    }

    /* load
    3 reg_a reg_b: load the 32-bit content from memory at address pointed by register reg_b
    into register reg_a using little-endian representation. */
    fn load(&mut self) -> Result<bool, MachineError> {
        let adr = &mut self.reg[IP];
        if (*adr as usize + 2)  >= MEMORY_SIZE {
            return Err(MachineError::InvalidMemoryAccess);
        }
        let reg_a = self.mem[*adr as usize + 1] as usize;
        let reg_b = self.mem[*adr as usize + 2] as usize;
        if reg_b >= NREGS || reg_a >= NREGS {
            return Err(MachineError::InvalidRegisterAccess);
        }
        /* I need to update adr here so that adr goes out of scope and can access other 
        registers directly */
        *adr += 3;
        if ((self.reg[reg_b] as usize)+3) >= MEMORY_SIZE {
            return Err(MachineError::InvalidMemoryAccess);
        }
        let src = &self.mem[self.reg[reg_b] as usize..(self.reg[reg_b]+4) as usize];
        self.reg[reg_a] = u32::from_le_bytes(src.try_into().unwrap());
        Ok(false)
    }

    /*loadimm
    4 reg_a L H: interpret H and L respectively as the high-order and the low-order bytes of
    a 16-bit signed value, sign-extend it to 32 bits, and store it into register reg_a. */
    fn loadimm(&mut self) -> Result<bool, MachineError> {
        let adr = &mut self.reg[IP];
        if (*adr as usize + 3)  >= MEMORY_SIZE {
            return Err(MachineError::InvalidMemoryAccess);
        }
        let reg_a = self.mem[*adr as usize + 1] as usize;
        let l = self.mem[*adr as usize + 2];
        let h = self.mem[*adr as usize + 3];
        if reg_a >= NREGS {
            return Err(MachineError::InvalidRegisterAccess);
        }
        /* I need to update adr here so that adr goes out of scope and can access other 
        registers directly */
        *adr += 4;
        let value = i16::from_le_bytes([l,h]);
        self.reg[reg_a] = value as u32;
        Ok(false)
    }

    /*sub
    5 reg_a reg_b reg_c: store the content of register reg_b minus the content of register reg_c into 
    register reg_a
    Arithmetic wraps around in case of overflow. For example, 0 - 1 returns 0xffffffff, 
    and 0 - 0xffffffff returns 1. */
    fn sub(&mut self) -> Result<bool, MachineError> {
        let adr = &mut self.reg[IP];
        if (*adr as usize + 3)  >= MEMORY_SIZE {
            return Err(MachineError::InvalidMemoryAccess);
        }
        let reg_a = self.mem[*adr as usize + 1] as usize;
        let reg_b = self.mem[*adr as usize + 2] as usize;
        let reg_c = self.mem[*adr as usize + 3] as usize;
        if reg_a >= NREGS || reg_b >= NREGS || reg_c >= NREGS {
            return Err(MachineError::InvalidRegisterAccess);
        }
        /* I need to update adr here so that adr goes out of scope and can access other 
        registers directly */
        *adr += 4;
        self.reg[reg_a] = self.reg[reg_b].wrapping_sub(self.reg[reg_c]);
        Ok(false)
    }

    /*out
    6 reg_a: output the character whose unicode value is stored in the 8 low bits of register reg_a. */
    fn out<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        let adr = &mut self.reg[IP];
        if (*adr as usize + 1) >= MEMORY_SIZE {
            return Err(MachineError::InvalidMemoryAccess);
        }
        let reg_a = self.mem[*adr as usize + 1] as usize;
        if reg_a >= NREGS {
            return Err(MachineError::InvalidRegisterAccess);
        }
        /* I need to update adr here so that adr goes out of scope and can access other 
        registers directly */
        *adr += 2;
        let c: char = self.reg[reg_a] as u8 as char;
        write!(fd, "{}", c).map_err(|e| MachineError::IOError(e))?;
        Ok(false)
    }

    /*exit
    7: exit the current program */
    fn exit(&mut self) -> Result<bool, MachineError> {
        let adr = &mut self.reg[IP];
        *adr += 1;
        Ok(true)
    }

    /*out number
    8 reg_a: output the signed number stored in register reg_a in decimal.*/
    fn out_number<T: Write>(&mut self, fd: &mut T) -> Result<bool, MachineError> {
        let adr = &mut self.reg[IP];
        if (*adr as usize + 1) >= MEMORY_SIZE {
            return Err(MachineError::InvalidMemoryAccess);
        }
        let reg_a = self.mem[*adr as usize + 1] as usize;
        if reg_a >= NREGS {
            return Err(MachineError::InvalidRegisterAccess);
        }
        /* I need to update adr here so that adr goes out of scope and can access other 
        registers directly */
        *adr += 2;
        write!(fd, "{}", self.reg[reg_a] as i32).map_err(|e| MachineError::IOError(e))?;
        Ok(false)
    }

}
