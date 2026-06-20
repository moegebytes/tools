use std::io::Cursor;

use anyhow::{Context, Result};

use crate::utils::io::*;

use super::opcode::*;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Instruction {
  pub offset: u32,
  pub opcode: Opcode,
  pub operand: Operand,
}

pub(crate) fn from_bytecode(mut cur: Cursor<&[u8]>) -> Result<Vec<Instruction>> {
  let mut instructions = Vec::new();

  while (cur.position() as usize) < cur.get_ref().len() {
    let offset = cur.position() as u32;
    let opcode = decode_opcode(read_u8(&mut cur)?).with_context(|| format!("at offset 0x{:06X}", offset))?;
    let operand = match opcode {
      Opcode::InitStack => Operand::InitStack {
        arg_count: read_u8(&mut cur)?,
        local_count: read_u8(&mut cur)?,
      },
      Opcode::Call | Opcode::Jmp | Opcode::Jz => Operand::Address(read_u32_le(&mut cur)?),
      Opcode::SysCall => Operand::U16(read_u16_le(&mut cur)?),
      Opcode::PushIntI32 => Operand::Int(read_i32_le(&mut cur)?, IntEncoding::I32),
      Opcode::PushIntI16 => Operand::Int(read_i16_le(&mut cur)? as i32, IntEncoding::I16),
      Opcode::PushIntI8 => Operand::Int(read_i8(&mut cur)? as i32, IntEncoding::I8),
      Opcode::PushFloat => Operand::Float(read_f32_le(&mut cur)?),
      Opcode::PushString => {
        let len = read_u8(&mut cur)? as usize;
        Operand::String(read_bytes(&mut cur, len)?)
      }
      Opcode::PushGlbvar | Opcode::PushGlbvarTable | Opcode::PopGlbvar | Opcode::PopGlbvarTable => {
        Operand::U16(read_u16_le(&mut cur)?)
      }
      Opcode::PushStkvar | Opcode::PushStkvarTable | Opcode::PopStkvar | Opcode::PopStkvarTable => {
        Operand::I8(read_i8(&mut cur)?)
      }
      _ => Operand::None,
    };

    instructions.push(Instruction {
      offset,
      opcode,
      operand,
    });
  }

  Ok(instructions)
}
