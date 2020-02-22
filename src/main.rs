


const CELL_MEMORY_SIZE:usize = 128;
const CELL_NUM_REG:usize = 8;
const SUBROUTINE_RETURN_REGISTER:usize = 7;

const OPCODE_SHIFT:usize = 12;
const DEST_REG_SHIFT:usize = 9;
const SRC1_SHIFT:usize = 6;
const IMMEDIATE_FLAG_SHIFT:usize = 5;
const NZP_SHIFT:usize = 9;
const SINGLE_SRC_REG_SHIFT:usize = 9;
const JUMP_FLAG_SHIFT:usize = 11;
const BASE_REG_SHIFT:usize = 6;

const DEST_REG_MASK:u16 = 0x0FFF;
const SRC1_MASK:u16 = 0x01FF;
const IMMEDIATE_FLAG_MASK:u16 = 0x003F;
const VAL5_MASK:u16 = 0x001F;
const SRC2_MASK:u16 = 0x0007;
const PCOFFSET6_MASK:u16 = 0x003F;
const PCOFFSET9_MASK:u16 = 0x01FF;
const PCOFFSET11_MASK:u16 = 0x07FF;
const SINGLE_SRC_REG_MASK:u16 = 0x0FFF;
const JUMP_FLAG_MASK:u16 = 0x0FFF;
const BASE_REG_MASK:u16 = 0x01FF;

#[derive(PartialEq)]
enum Sign {
    Z,
    N,
    P
}

/* Underscore at the end is a reminder that that opcode has different modes, most commands are the
  same as in the LC-3 with a few exceptions that are desccribed in a comment above their
  implementation functions.
*/
#[repr(u16)]
enum Opcode {
    BR = 0,
    ADD_,
    LD,
    ST,
    JSR_,
    AND_,
    LDR,
    STR,
    LOOK,
    NOT,
    LDI,
    STI,
    JMP,
    SETBC,
    LEA,
    SNDACC_
}

struct Cell {
    pc:u16,
    last_sign:Sign,
    reg:Vec<u16>,
    mem:Vec<u16>
}

impl Cell {
    fn to_string(&self) -> String {
        return format!("Num reg: {}, Mem size: {}",self.reg.capacity(),self.mem.capacity());
    }

    fn get_reg_val(&self,index:usize) -> u16 {
        return self.reg[index];
    }
}

impl Cell {
    fn run_instr(&mut self, addr:u16) {
        let instr = self.mem[(addr as usize) % self.mem.len()];
        let opcode = instr >> OPCODE_SHIFT;
        if opcode == Opcode::BR as u16 {
            self.branch(instr);
        }
        else if opcode == Opcode::ADD_ as u16 {
            self.add(instr);
        }
        else if opcode == Opcode::LD as u16 {
            self.load(instr);
        }
        else if opcode == Opcode::ST as u16 {
            self.store(instr);
        }
        else if opcode == Opcode::JSR_ as u16 {
            self.jump_subroutine(instr);
        }
        else if opcode == Opcode::AND_ as u16 {
            self.and(instr);
        }
        else if opcode == Opcode::LDR as u16 {
            self.load_reg(instr);
        }
        else if opcode == Opcode::STR as u16 {
            self.store_register(instr);
        }
        else if opcode == Opcode::LOOK as u16 {
            // ### TODO: MAKE LOOK AT FUNCTION ### //
        }
        else if opcode == Opcode::NOT as u16 {
            self.not(instr);
        }
        else if opcode == Opcode::LDI as u16 {
            self.load_indirect(instr);
        }
        else if opcode == Opcode::STI as u16 {
            self.store_indirect(instr);
        }
        else if opcode == Opcode::JMP as u16 {
            self.jump(instr);
        }
        else if opcode == Opcode::SETBC as u16 {
            // ### TODO: MAKE SET BROADCAST FUNCTION ### //
        }
        else if opcode == Opcode::LEA as u16 {
            self.load_effective_addr(instr);
        }
        else if opcode == Opcode::SNDACC_ as u16 {
            // ### TODO: MAKE SEND ACCEPT FUNCTION ### //
        }

    }

    fn update_sign(&mut self, dest:u16) {
        let result:i16 = self.reg[dest as usize] as i16;
        if result < 0 {
            self.last_sign = Sign::N;
        }
        else if result > 0 {
            self.last_sign = Sign::P;
        }
        else {
            self.last_sign = Sign::Z;
        }
    }


    fn branch(&mut self, instr:u16) {
        // Flags for whether to branch if last sign is negative, zero or positive. Can be all, none or some combination.
        let nzp = instr >> NZP_SHIFT;
        let offset = instr & PCOFFSET9_MASK;
        if (nzp % 2 == 1 && self.last_sign == Sign::P) ||
           ((nzp >> 1) % 2 == 1 && self.last_sign == Sign::Z) ||
           ((nzp >> 2) % 2 == 1 && self.last_sign == Sign::N) {
            self.pc = self.pc.wrapping_add(offset) % CELL_MEMORY_SIZE as u16;
        }

    }


    fn add(&mut self,instr:u16) {
        let dest = (instr & DEST_REG_MASK) >> DEST_REG_SHIFT;
        let src1 = (instr & SRC1_MASK) >> SRC1_SHIFT;
        let flag = (instr & IMMEDIATE_FLAG_MASK) >> IMMEDIATE_FLAG_SHIFT;

        if flag == 0 {
            let src2 = instr & SRC2_MASK;
            self.reg[dest as usize] = self.reg[src1 as usize].wrapping_add(self.reg[src2 as usize]);
        }
        else {
            let val = instr & VAL5_MASK;
            self.reg[dest as usize] = self.reg[src1 as usize].wrapping_add(val);
        }

        self.update_sign(dest);
    }


    fn load (&mut self, instr:u16) {
        let dest = (instr &DEST_REG_MASK) >> DEST_REG_SHIFT;
        let offset = instr & PCOFFSET9_MASK;

        self.reg[dest as usize] = self.mem[(self.pc.wrapping_add(offset) % CELL_MEMORY_SIZE as u16) as usize];
        self.update_sign(dest);
    }


    fn store(&mut self, instr:u16) {
        let src = (instr & SINGLE_SRC_REG_MASK) >> SINGLE_SRC_REG_SHIFT;
        let offset = instr & PCOFFSET9_MASK;
        self.mem[(self.pc.wrapping_add(offset) % CELL_MEMORY_SIZE as u16) as usize] = self.reg[src as usize];
    }

    fn jump_subroutine(&mut self, instr:u16) {
        self.reg[SUBROUTINE_RETURN_REGISTER] = self.pc;

        let jump_flag = (instr & JUMP_FLAG_MASK) >> JUMP_FLAG_SHIFT;
        if jump_flag == 1 {
            let offset = instr & PCOFFSET11_MASK;
            self.pc = self.pc.wrapping_add(offset) % CELL_MEMORY_SIZE as u16;
        }
        else {
            let base = (instr & BASE_REG_MASK) >> BASE_REG_SHIFT;
            self.pc = base % CELL_MEMORY_SIZE as u16;
        }
    }

    fn and(&mut self, instr:u16) {
        let dest = (instr & DEST_REG_MASK) >> DEST_REG_SHIFT;
        let src1 = (instr & SRC1_MASK) >> SRC1_SHIFT;
        let flag = (instr & IMMEDIATE_FLAG_MASK) >> IMMEDIATE_FLAG_SHIFT;

        if flag == 0 {
            let src2 = instr & SRC2_MASK;
            self.reg[dest as usize] = self.reg[src1 as usize] & self.reg[src2 as usize];
        }
        else {
            let val = instr & VAL5_MASK;
            self.reg[dest as usize] = self.reg[src1 as usize] & val;
        }

        self.update_sign(dest);
    }

    fn load_reg(&mut self, instr:u16) {
        let dest = (instr & DEST_REG_MASK) >> DEST_REG_SHIFT;
        let base = (instr & BASE_REG_MASK) >> BASE_REG_SHIFT;
        let offset = instr & PCOFFSET6_MASK;

        self.reg[dest as usize] = self.mem[(base.wrapping_add(offset) as usize) % CELL_MEMORY_SIZE];
        self.update_sign(dest);
    }

    fn not(&mut self, instr:u16) {
        let dest = (instr & DEST_REG_MASK) >> DEST_REG_SHIFT;
        let src = (instr & SRC1_MASK) >> SRC1_SHIFT;

        self.reg[dest as usize] = !self.reg[src as usize];
        self.update_sign(dest);
    }

    fn load_indirect(&mut self, instr:u16) {
        let dest = (instr & DEST_REG_MASK) >> DEST_REG_SHIFT;
        let offset = instr & PCOFFSET9_MASK;

        let mem_location = self.mem[(self.pc.wrapping_add(offset) as usize) % CELL_MEMORY_SIZE];
        self.reg[dest as usize] = self.mem[mem_location as usize % CELL_MEMORY_SIZE];
        self.update_sign(dest);
    }

    fn store_indirect(&mut self, instr:u16) {
        let src = (instr & SINGLE_SRC_REG_MASK) >> SINGLE_SRC_REG_SHIFT;
        let offset = instr & PCOFFSET9_MASK;

        let mem_location = self.mem[(self.pc.wrapping_add(offset) as usize) % CELL_MEMORY_SIZE];
        self.mem[mem_location as usize % CELL_MEMORY_SIZE] = self.reg[src as usize];
    }

    fn jump(&mut self, instr:u16) {
        let base = (instr & BASE_REG_MASK) >> BASE_REG_SHIFT;
        self.pc = base % CELL_MEMORY_SIZE as u16;
    }

    fn load_effective_addr(&mut self, instr:u16) {
        let dest = (instr & DEST_REG_MASK) >> DEST_REG_SHIFT;
        let offset = instr & PCOFFSET9_MASK;

        self.reg[dest as usize] = self.pc.wrapping_add(offset) % CELL_MEMORY_SIZE as u16;
        self.update_sign(dest);
    }

    fn store_register(&mut self, instr:u16) {
        let src = (instr & SINGLE_SRC_REG_MASK) >> SINGLE_SRC_REG_SHIFT;
        let base = (instr & BASE_REG_MASK) >> BASE_REG_SHIFT;
        let offset = instr & PCOFFSET6_MASK;

        self.mem[base.wrapping_add(offset) as usize % CELL_MEMORY_SIZE] = self.reg[src as usize];
    }



}



fn main() {
    let mut test:Cell = Cell {pc:0, last_sign:Sign::Z, mem: vec![0;CELL_MEMORY_SIZE], reg:vec![0;CELL_NUM_REG]};
    for i in 0..10000 {
        test.add(0x102F);
        println!("{}: {}",i,test.get_reg_val(0));
    }
}
