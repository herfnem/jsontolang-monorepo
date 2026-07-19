use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MemoryAddr {
    Immediate(i32),
    Register(usize),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operand {
    Register(usize),    // 0 to 7 (r0-r7), 8 (sp), 9 (pc)
    Immediate(i32),     // 123
    Memory(MemoryAddr), // [123] or [r1]
    Label(String),      // label_name
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InstructionType {
    Mov,
    Add,
    Sub,
    Mul,
    Div,
    Cmp,
    Jmp,
    Je,
    Jne,
    Jg,
    Jl,
    Push,
    Pop,
    Call,
    Ret,
    Halt,
    Nop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instruction {
    pub op: InstructionType,
    pub operands: Vec<Operand>,
    pub line_num: usize,
    pub original_text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParseResult {
    pub instructions: Vec<Instruction>,
    pub labels: HashMap<String, usize>,
    pub errors: Vec<ParseError>,
}

fn parse_register(s: &str) -> Option<usize> {
    let s_lower = s.to_ascii_lowercase();
    match s_lower.as_str() {
        "r0" => Some(0),
        "r1" => Some(1),
        "r2" => Some(2),
        "r3" => Some(3),
        "r4" => Some(4),
        "r5" => Some(5),
        "r6" => Some(6),
        "r7" => Some(7),
        "sp" => Some(8),
        "pc" => Some(9),
        _ => s_lower
            .strip_prefix('r')
            .and_then(|stripped| stripped.parse::<usize>().ok())
            .filter(|&idx| idx < 8),
    }
}

fn parse_operand(s: &str) -> Result<Operand, String> {
    let s = s.trim();
    if s.is_empty() {
        return Err("Empty operand".to_string());
    }

    // Check memory operand [addr]
    if s.starts_with('[') && s.ends_with(']') {
        let inside = &s[1..s.len() - 1].trim();
        if let Some(reg_idx) = parse_register(inside) {
            return Ok(Operand::Memory(MemoryAddr::Register(reg_idx)));
        }
        if let Ok(val) = inside.parse::<i32>() {
            return Ok(Operand::Memory(MemoryAddr::Immediate(val)));
        }
        // Also support hex in memory address e.g. [0x10]
        if let Some(val) = inside.strip_prefix("0x")
            .or_else(|| inside.strip_prefix("0X"))
            .and_then(|digits| i32::from_str_radix(digits, 16).ok())
        {
            return Ok(Operand::Memory(MemoryAddr::Immediate(val)));
        }
        return Err(format!("Invalid memory address: {}", inside));
    }

    // Check register
    if let Some(reg_idx) = parse_register(s) {
        return Ok(Operand::Register(reg_idx));
    }

    // Check immediate
    if let Ok(val) = s.parse::<i32>() {
        return Ok(Operand::Immediate(val));
    }
    if let Some(val) = s.strip_prefix("0x")
        .or_else(|| s.strip_prefix("0X"))
        .and_then(|digits| i32::from_str_radix(digits, 16).ok())
    {
        return Ok(Operand::Immediate(val));
    }

    // Otherwise, assume it is a label
    // Validate label format (only alphanumeric and underscore)
    if s.chars().all(|c| c.is_alphanumeric() || c == '_') {
        Ok(Operand::Label(s.to_string()))
    } else {
        Err(format!("Invalid operand format: {}", s))
    }
}

pub fn parse_program(source: &str) -> ParseResult {
    let mut instructions = Vec::new();
    let mut labels = HashMap::new();
    let mut errors = Vec::new();

    // First pass: extract labels and clean lines
    // We want to keep track of the lines so we can report errors on correct line numbers.
    let mut cleaned_lines = Vec::new();

    for (line_idx, raw_line) in source.lines().enumerate() {
        let line_num = line_idx + 1;
        // Strip comments starting with ';'
        let comment_idx = raw_line.find(';');
        let without_comment = match comment_idx {
            Some(idx) => &raw_line[..idx],
            None => raw_line,
        };
        let trimmed = without_comment.trim();

        if trimmed.is_empty() {
            continue;
        }

        // Check if this line is just a label (ends with ':') or starts with a label
        if let Some(stripped) = trimmed.strip_suffix(':') {
            let label_name = stripped.trim();
            if label_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                cleaned_lines.push((
                    line_num,
                    raw_line.to_string(),
                    Some(label_name.to_string()),
                    None,
                ));
            } else {
                errors.push(ParseError {
                    line: line_num,
                    message: format!("Invalid label name: {}", label_name),
                });
            }
        } else if let Some(colon_idx) = trimmed.find(':') {
            // Label on the same line as an instruction, e.g. "loop: MOV r0, r1"
            let label_name = trimmed[..colon_idx].trim();
            let rest_inst = trimmed[colon_idx + 1..].trim();
            if label_name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                cleaned_lines.push((
                    line_num,
                    raw_line.to_string(),
                    Some(label_name.to_string()),
                    Some(rest_inst.to_string()),
                ));
            } else {
                errors.push(ParseError {
                    line: line_num,
                    message: format!("Invalid label name: {}", label_name),
                });
                cleaned_lines.push((
                    line_num,
                    raw_line.to_string(),
                    None,
                    Some(rest_inst.to_string()),
                ));
            }
        } else {
            cleaned_lines.push((
                line_num,
                raw_line.to_string(),
                None,
                Some(trimmed.to_string()),
            ));
        }
    }

    // Pass 1.5: Register all label indices
    let mut instruction_counter = 0;
    for (_, _, opt_label, opt_inst) in &cleaned_lines {
        if let Some(label) = opt_label {
            labels.insert(label.clone(), instruction_counter);
        }
        if opt_inst.is_some() {
            instruction_counter += 1;
        }
    }

    // Pass 2: Parse instructions
    for (line_num, original_text, _, opt_inst) in cleaned_lines {
        let inst_str = match opt_inst {
            Some(s) => s,
            None => continue, // Just a label line
        };

        // Split instruction by first whitespace to get opcode and operands
        let parts: Vec<&str> = inst_str.splitn(2, |c: char| c.is_whitespace()).collect();
        let op_str = parts[0].trim();
        let op_type = match op_str.to_ascii_uppercase().as_str() {
            "MOV" => InstructionType::Mov,
            "ADD" => InstructionType::Add,
            "SUB" => InstructionType::Sub,
            "MUL" => InstructionType::Mul,
            "DIV" => InstructionType::Div,
            "CMP" => InstructionType::Cmp,
            "JMP" => InstructionType::Jmp,
            "JE" => InstructionType::Je,
            "JNE" => InstructionType::Jne,
            "JG" => InstructionType::Jg,
            "JL" => InstructionType::Jl,
            "PUSH" => InstructionType::Push,
            "POP" => InstructionType::Pop,
            "CALL" => InstructionType::Call,
            "RET" => InstructionType::Ret,
            "HALT" => InstructionType::Halt,
            "NOP" => InstructionType::Nop,
            other => {
                errors.push(ParseError {
                    line: line_num,
                    message: format!("Unknown instruction: {}", other),
                });
                continue;
            }
        };

        let mut operands = Vec::new();
        if parts.len() > 1 {
            let operands_str = parts[1];
            // Split by comma
            let raw_operands: Vec<&str> = operands_str.split(',').collect();
            let mut parse_failed = false;
            for raw_op in raw_operands {
                let op_trimmed = raw_op.trim();
                if op_trimmed.is_empty() {
                    continue;
                }
                match parse_operand(op_trimmed) {
                    Ok(operand) => operands.push(operand),
                    Err(err_msg) => {
                        errors.push(ParseError {
                            line: line_num,
                            message: err_msg,
                        });
                        parse_failed = true;
                        break;
                    }
                }
            }
            if parse_failed {
                continue;
            }
        }

        // Validate operand count and types for safety
        let expected_count = match op_type {
            InstructionType::Ret | InstructionType::Halt | InstructionType::Nop => 0,
            InstructionType::Jmp
            | InstructionType::Je
            | InstructionType::Jne
            | InstructionType::Jg
            | InstructionType::Jl
            | InstructionType::Push
            | InstructionType::Pop
            | InstructionType::Call => 1,
            InstructionType::Mov
            | InstructionType::Add
            | InstructionType::Sub
            | InstructionType::Mul
            | InstructionType::Div
            | InstructionType::Cmp => 2,
        };

        if operands.len() != expected_count {
            errors.push(ParseError {
                line: line_num,
                message: format!(
                    "Instruction {} expects {} operands, but found {}",
                    op_str.to_ascii_uppercase(),
                    expected_count,
                    operands.len()
                ),
            });
            continue;
        }

        instructions.push(Instruction {
            op: op_type,
            operands,
            line_num,
            original_text,
        });
    }

    ParseResult {
        instructions,
        labels,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let code = "
            MOV r0, 10
            MOV r1, r0
            loop:
            ADD r0, [0x10]
            CMP r0, r1
            JNE loop
            HALT
        ";
        let res = parse_program(code);
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        assert_eq!(res.instructions.len(), 6);
        assert_eq!(res.labels.get("loop"), Some(&2));
        assert_eq!(res.instructions[0].op, InstructionType::Mov);
        assert_eq!(res.instructions[2].op, InstructionType::Add);
    }
}
