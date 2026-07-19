use parser::{Instruction, InstructionType, MemoryAddr, Operand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const MEMORY_SIZE: usize = 256;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuState {
    pub registers: [i32; 10], // r0-r7 (0-7), sp (8), pc (9)
    pub memory: Vec<i32>,
    pub equal_flag: bool,
    pub greater_flag: bool,
    pub less_flag: bool,
    pub halted: bool,
    pub steps_count: usize,
    pub error: Option<String>,
}

pub struct Vm {
    pub state: CpuState,
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<String, usize>,
}

impl Vm {
    pub fn new(instructions: Vec<Instruction>, labels: HashMap<String, usize>) -> Self {
        let mut registers = [0; 10];
        // Initialize Stack Pointer to the end of memory
        registers[8] = MEMORY_SIZE as i32;
        // Initialize PC to 0
        registers[9] = 0;

        Vm {
            state: CpuState {
                registers,
                memory: vec![0; MEMORY_SIZE],
                equal_flag: false,
                greater_flag: false,
                less_flag: false,
                halted: false,
                steps_count: 0,
                error: None,
            },
            instructions,
            labels,
        }
    }

    fn resolve_label(&self, name: &str) -> Result<i32, String> {
        self.labels
            .get(name)
            .map(|&idx| idx as i32)
            .ok_or_else(|| format!("Label '{}' not found", name))
    }

    fn get_val(&self, op: &Operand) -> Result<i32, String> {
        match op {
            Operand::Register(idx) => {
                if *idx < 10 {
                    Ok(self.state.registers[*idx])
                } else {
                    Err(format!("Invalid register index: {}", idx))
                }
            }
            Operand::Immediate(val) => Ok(*val),
            Operand::Memory(addr) => {
                let physical_addr = match addr {
                    MemoryAddr::Immediate(val) => *val,
                    MemoryAddr::Register(idx) => {
                        if *idx < 10 {
                            self.state.registers[*idx]
                        } else {
                            return Err(format!("Invalid memory register index: {}", idx));
                        }
                    }
                };

                if physical_addr < 0 || physical_addr >= MEMORY_SIZE as i32 {
                    Err(format!(
                        "Memory address out of bounds: {} (Max: {})",
                        physical_addr,
                        MEMORY_SIZE - 1
                    ))
                } else {
                    Ok(self.state.memory[physical_addr as usize])
                }
            }
            Operand::Label(name) => self.resolve_label(name),
        }
    }

    fn set_val(&mut self, op: &Operand, val: i32) -> Result<(), String> {
        match op {
            Operand::Register(idx) => {
                if *idx == 9 {
                    return Err(
                        "Cannot directly write to PC register using this instruction".to_string(),
                    );
                }
                if *idx < 10 {
                    self.state.registers[*idx] = val;
                    Ok(())
                } else {
                    Err(format!("Invalid register index: {}", idx))
                }
            }
            Operand::Memory(addr) => {
                let physical_addr = match addr {
                    MemoryAddr::Immediate(val) => *val,
                    MemoryAddr::Register(idx) => {
                        if *idx < 10 {
                            self.state.registers[*idx]
                        } else {
                            return Err(format!("Invalid memory register index: {}", idx));
                        }
                    }
                };

                if physical_addr < 0 || physical_addr >= MEMORY_SIZE as i32 {
                    Err(format!(
                        "Memory address out of bounds: {} (Max: {})",
                        physical_addr,
                        MEMORY_SIZE - 1
                    ))
                } else {
                    self.state.memory[physical_addr as usize] = val;
                    Ok(())
                }
            }
            Operand::Immediate(_) => Err("Cannot write to an immediate value".to_string()),
            Operand::Label(_) => Err("Cannot write to a label".to_string()),
        }
    }

    pub fn step(&mut self) -> Result<CpuState, String> {
        if self.state.halted {
            return Ok(self.state.clone());
        }

        let pc = self.state.registers[9] as usize;
        if pc >= self.instructions.len() {
            self.state.halted = true;
            return Ok(self.state.clone());
        }

        let inst = self.instructions[pc].clone();
        self.state.steps_count += 1;

        // Auto-increment PC. Jump instructions can override this.
        self.state.registers[9] += 1;

        let result = match inst.op {
            InstructionType::Mov => {
                let src_val = self.get_val(&inst.operands[1])?;
                self.set_val(&inst.operands[0], src_val)
            }
            InstructionType::Add => {
                let dest_val = self.get_val(&inst.operands[0])?;
                let src_val = self.get_val(&inst.operands[1])?;
                self.set_val(&inst.operands[0], dest_val.wrapping_add(src_val))
            }
            InstructionType::Sub => {
                let dest_val = self.get_val(&inst.operands[0])?;
                let src_val = self.get_val(&inst.operands[1])?;
                self.set_val(&inst.operands[0], dest_val.wrapping_sub(src_val))
            }
            InstructionType::Mul => {
                let dest_val = self.get_val(&inst.operands[0])?;
                let src_val = self.get_val(&inst.operands[1])?;
                self.set_val(&inst.operands[0], dest_val.wrapping_mul(src_val))
            }
            InstructionType::Div => {
                let dest_val = self.get_val(&inst.operands[0])?;
                let src_val = self.get_val(&inst.operands[1])?;
                if src_val == 0 {
                    Err("Division by zero".to_string())
                } else {
                    self.set_val(&inst.operands[0], dest_val / src_val)
                }
            }
            InstructionType::Cmp => {
                let val1 = self.get_val(&inst.operands[0])?;
                let val2 = self.get_val(&inst.operands[1])?;
                self.state.equal_flag = val1 == val2;
                self.state.greater_flag = val1 > val2;
                self.state.less_flag = val1 < val2;
                Ok(())
            }
            InstructionType::Jmp => {
                let target = self.get_val(&inst.operands[0])?;
                if target < 0 || target > self.instructions.len() as i32 {
                    Err(format!("Jump target out of bounds: {}", target))
                } else {
                    self.state.registers[9] = target;
                    Ok(())
                }
            }
            InstructionType::Je => {
                if self.state.equal_flag {
                    let target = self.get_val(&inst.operands[0])?;
                    if target < 0 || target > self.instructions.len() as i32 {
                        return Err(format!("Jump target out of bounds: {}", target));
                    }
                    self.state.registers[9] = target;
                }
                Ok(())
            }
            InstructionType::Jne => {
                if !self.state.equal_flag {
                    let target = self.get_val(&inst.operands[0])?;
                    if target < 0 || target > self.instructions.len() as i32 {
                        return Err(format!("Jump target out of bounds: {}", target));
                    }
                    self.state.registers[9] = target;
                }
                Ok(())
            }
            InstructionType::Jg => {
                if self.state.greater_flag {
                    let target = self.get_val(&inst.operands[0])?;
                    if target < 0 || target > self.instructions.len() as i32 {
                        return Err(format!("Jump target out of bounds: {}", target));
                    }
                    self.state.registers[9] = target;
                }
                Ok(())
            }
            InstructionType::Jl => {
                if self.state.less_flag {
                    let target = self.get_val(&inst.operands[0])?;
                    if target < 0 || target > self.instructions.len() as i32 {
                        return Err(format!("Jump target out of bounds: {}", target));
                    }
                    self.state.registers[9] = target;
                }
                Ok(())
            }
            InstructionType::Push => {
                let val = self.get_val(&inst.operands[0])?;
                let sp = self.state.registers[8];
                let next_sp = sp - 1;
                if next_sp < 0 || next_sp >= MEMORY_SIZE as i32 {
                    Err("Stack overflow".to_string())
                } else {
                    self.state.registers[8] = next_sp;
                    self.state.memory[next_sp as usize] = val;
                    Ok(())
                }
            }
            InstructionType::Pop => {
                let sp = self.state.registers[8];
                if sp < 0 || sp >= MEMORY_SIZE as i32 {
                    Err("Stack underflow".to_string())
                } else {
                    let val = self.state.memory[sp as usize];
                    self.state.registers[8] = sp + 1;
                    self.set_val(&inst.operands[0], val)
                }
            }
            InstructionType::Call => {
                let target = self.get_val(&inst.operands[0])?;
                if target < 0 || target > self.instructions.len() as i32 {
                    return Err(format!("Call target out of bounds: {}", target));
                }
                let sp = self.state.registers[8];
                let next_sp = sp - 1;
                if next_sp < 0 || next_sp >= MEMORY_SIZE as i32 {
                    Err("Stack overflow on Call".to_string())
                } else {
                    self.state.registers[8] = next_sp;
                    self.state.memory[next_sp as usize] = self.state.registers[9]; // push return address
                    self.state.registers[9] = target; // jump to target
                    Ok(())
                }
            }
            InstructionType::Ret => {
                let sp = self.state.registers[8];
                if sp < 0 || sp >= MEMORY_SIZE as i32 {
                    Err("Stack underflow on Return".to_string())
                } else {
                    let ret_addr = self.state.memory[sp as usize];
                    self.state.registers[8] = sp + 1;
                    if ret_addr < 0 || ret_addr > self.instructions.len() as i32 {
                        Err(format!("Return address out of bounds: {}", ret_addr))
                    } else {
                        self.state.registers[9] = ret_addr;
                        Ok(())
                    }
                }
            }
            InstructionType::Halt => {
                self.state.halted = true;
                Ok(())
            }
            InstructionType::Nop => Ok(()),
        };

        if let Err(err_msg) = result {
            let full_err = format!("Error at line {}: {}", inst.line_num, err_msg);
            self.state.error = Some(full_err.clone());
            self.state.halted = true;
            Err(err_msg)
        } else {
            // If we run past the end of the instructions, halt
            if self.state.registers[9] < 0
                || self.state.registers[9] >= self.instructions.len() as i32
            {
                self.state.halted = true;
            }
            Ok(self.state.clone())
        }
    }

    pub fn run(&mut self, max_steps: usize) -> CpuState {
        for _ in 0..max_steps {
            if self.state.halted {
                break;
            }
            if self.step().is_err() {
                break;
            }
        }
        self.state.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parser::parse_program;

    #[test]
    fn test_factorial() {
        // Simple assembler factorial algorithm
        let code = "
            MOV r0, 5    ; n = 5
            MOV r1, 1    ; acc = 1
            
            loop:
            CMP r0, 1
            JL done
            MUL r1, r0
            SUB r0, 1
            JMP loop

            done:
            HALT
        ";
        let parsed = parse_program(code);
        assert!(
            parsed.errors.is_empty(),
            "Parse errors: {:?}",
            parsed.errors
        );

        let mut vm = Vm::new(parsed.instructions, parsed.labels);
        let final_state = vm.run(100);

        assert!(
            final_state.error.is_none(),
            "VM error: {:?}",
            final_state.error
        );
        assert_eq!(final_state.registers[1], 120); // acc should be 5! = 120
    }
}
