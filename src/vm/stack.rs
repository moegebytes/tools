use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};

use anyhow::{bail, Result};

use super::instruction::Instruction;
use super::opcode::*;
use super::Syscall;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum StackValue {
  Unknown,
  Int { value: i32, producer: Option<u32> },
}

pub(crate) enum ProblemMode {
  Warn,
  Strict,
}

pub(crate) struct InstructionFrame<'a> {
  pub instr: &'a Instruction,
  pub stack: &'a [StackValue],
}

#[derive(Clone, PartialEq)]
struct StackState {
  values: Vec<StackValue>,
}

enum Flow {
  Next,
  Jump(u32),
  Branch(u32),
  Return,
}

struct Emulator<'a, F>
where
  F: FnMut(InstructionFrame<'_>) -> Result<()>,
{
  instructions: &'a [Instruction],
  function_args: &'a BTreeMap<u32, usize>,
  syscalls: &'a [Syscall],
  index_by_offset: &'a HashMap<u32, usize>,
  mode: &'a ProblemMode,
  on_instruction: &'a mut F,
}

struct FunctionRun {
  start_idx: usize,
  end_idx: usize,
  states: HashMap<usize, StackState>,
  worklist: VecDeque<usize>,
}

impl FunctionRun {
  fn new(start_idx: usize, end_idx: usize) -> Self {
    let mut states = HashMap::new();
    states.insert(start_idx, StackState { values: Vec::new() });
    Self {
      start_idx,
      end_idx,
      states,
      worklist: VecDeque::from([start_idx]),
    }
  }
}

pub(crate) fn emulate<F>(
  instructions: &[Instruction],
  functions: &BTreeSet<u32>,
  syscalls: &[Syscall],
  mode: ProblemMode,
  mut on_instruction: F,
) -> Result<()>
where
  F: FnMut(InstructionFrame<'_>) -> Result<()>,
{
  let index_by_offset: HashMap<u32, usize> = instructions
    .iter()
    .enumerate()
    .map(|(i, instr)| (instr.offset, i))
    .collect();
  let function_args = function_args(instructions, functions);
  let function_starts: Vec<u32> = functions.iter().copied().collect();
  let mut emulator = Emulator {
    instructions,
    function_args: &function_args,
    syscalls,
    index_by_offset: &index_by_offset,
    mode: &mode,
    on_instruction: &mut on_instruction,
  };

  for (func_pos, &func_start) in function_starts.iter().enumerate() {
    let Some(&start_idx) = index_by_offset.get(&func_start) else {
      handle_problem(
        &mode,
        format!("function at 0x{:06X} is not an instruction boundary", func_start),
      )?;
      continue;
    };
    let end_idx = function_starts
      .get(func_pos + 1)
      .and_then(|offset| index_by_offset.get(offset))
      .copied()
      .unwrap_or(instructions.len());

    emulator.emulate_function(start_idx, end_idx)?;
  }

  Ok(())
}

fn function_args(instructions: &[Instruction], functions: &BTreeSet<u32>) -> BTreeMap<u32, usize> {
  instructions
    .iter()
    .filter_map(|instr| {
      if !functions.contains(&instr.offset) {
        return None;
      }
      match instr.operand {
        Operand::InitStack { arg_count, .. } => Some((instr.offset, arg_count as usize)),
        _ => None,
      }
    })
    .collect()
}

impl<F> Emulator<'_, F>
where
  F: FnMut(InstructionFrame<'_>) -> Result<()>,
{
  fn emulate_function(&mut self, start_idx: usize, end_idx: usize) -> Result<()> {
    let mut run = FunctionRun::new(start_idx, end_idx);

    while let Some(idx) = run.worklist.pop_front() {
      let Some(state) = run.states.get(&idx).cloned() else {
        continue;
      };
      let instr = &self.instructions[idx];
      let Some((next_state, flow)) = self.apply_instruction(instr, state)? else {
        continue;
      };

      match flow {
        Flow::Next => {
          if idx + 1 >= run.end_idx {
            handle_problem(
              self.mode,
              format!(
                "function at 0x{:06X} falls through without ret",
                self.instructions[run.start_idx].offset
              ),
            )?;
          } else {
            enqueue_state(idx + 1, next_state, &mut run, self.mode)?;
          }
        }
        Flow::Jump(target) => {
          self.enqueue_target(target, next_state, &mut run)?;
        }
        Flow::Branch(target) => {
          self.enqueue_target(target, next_state.clone(), &mut run)?;
          if idx + 1 >= run.end_idx {
            handle_problem(
              self.mode,
              format!("jz at 0x{:06X} can fall through past function end", instr.offset),
            )?;
          } else {
            enqueue_state(idx + 1, next_state, &mut run, self.mode)?;
          }
        }
        Flow::Return => {}
      }
    }

    Ok(())
  }

  fn enqueue_target(&self, target: u32, state: StackState, run: &mut FunctionRun) -> Result<()> {
    let Some(&target_idx) = self.index_by_offset.get(&target) else {
      handle_problem(
        self.mode,
        format!("jump target 0x{:06X} is not an instruction boundary", target),
      )?;
      return Ok(());
    };
    if target_idx < run.start_idx || target_idx >= run.end_idx {
      handle_problem(
        self.mode,
        format!(
          "jump target 0x{:06X} is outside function starting at 0x{:06X}",
          target, self.instructions[run.start_idx].offset
        ),
      )?;
      return Ok(());
    }
    enqueue_state(target_idx, state, run, self.mode)
  }

  fn apply_instruction(&mut self, instr: &Instruction, mut state: StackState) -> Result<Option<(StackState, Flow)>> {
    (self.on_instruction)(InstructionFrame {
      instr,
      stack: &state.values,
    })?;

    match instr.opcode {
      Opcode::Nop | Opcode::InitStack => {}
      Opcode::Call => {
        let target = address_operand(instr)?;
        let Some(&arg_count) = self.function_args.get(&target) else {
          handle_problem(
            self.mode,
            format!("call at 0x{:06X} targets non-function 0x{:06X}", instr.offset, target),
          )?;
          return Ok(None);
        };
        pop_values(&mut state, arg_count, instr.offset, "call arguments", self.mode)?;
      }
      Opcode::SysCall => {
        let arg_count = syscall_arg_count(instr, self.syscalls, self.mode)?;
        if state.values.len() < arg_count {
          handle_problem(
            self.mode,
            format!(
              "syscall arguments at 0x{:06X} need {} value(s), but stack has {}",
              instr.offset,
              arg_count,
              state.values.len()
            ),
          )?;
          state.values.clear();
          return Ok(None);
        }
        state.values.truncate(state.values.len() - arg_count);
      }
      Opcode::Ret => return Ok(Some((state, Flow::Return))),
      Opcode::Retv => {
        pop_values(&mut state, 1, instr.offset, "retv value", self.mode)?;
        return Ok(Some((state, Flow::Return)));
      }
      Opcode::Jmp => return Ok(Some((state, Flow::Jump(address_operand(instr)?)))),
      Opcode::Jz => {
        pop_values(&mut state, 1, instr.offset, "jz condition", self.mode)?;
        return Ok(Some((state, Flow::Branch(address_operand(instr)?))));
      }
      Opcode::PushNil
      | Opcode::PushTrue
      | Opcode::PushFloat
      | Opcode::PushString
      | Opcode::PushGlbvar
      | Opcode::PushReturn => state.values.push(StackValue::Unknown),
      Opcode::PushIntI32 | Opcode::PushIntI16 | Opcode::PushIntI8 => {
        let value = int_operand(instr)?;
        state.values.push(StackValue::Int {
          value,
          producer: Some(instr.offset),
        });
      }
      Opcode::PushStkvar => state.values.push(StackValue::Unknown),
      Opcode::PushGlbvarTable | Opcode::PushStkvarTable => {
        pop_values(&mut state, 1, instr.offset, "table key", self.mode)?;
        state.values.push(StackValue::Unknown);
      }
      Opcode::PushTop => {
        let Some(value) = state.values.last().cloned() else {
          handle_problem(
            self.mode,
            format!("push_top at 0x{:06X} reads an empty stack", instr.offset),
          )?;
          return Ok(None);
        };
        state.values.push(value);
      }
      Opcode::PopGlbvar | Opcode::PopStkvar => {
        pop_values(&mut state, 1, instr.offset, "store value", self.mode)?;
      }
      Opcode::PopGlbvarTable | Opcode::PopStkvarTable => {
        pop_values(&mut state, 2, instr.offset, "table store value and key", self.mode)?;
      }
      Opcode::Neg => {
        replace_top_unknown(&mut state, instr.offset, "neg operand", self.mode)?;
      }
      Opcode::Add
      | Opcode::Sub
      | Opcode::Mul
      | Opcode::Div
      | Opcode::Mod
      | Opcode::BitTest
      | Opcode::And
      | Opcode::Or
      | Opcode::SetE
      | Opcode::SetNe
      | Opcode::SetG
      | Opcode::SetGe
      | Opcode::SetL
      | Opcode::SetLe => {
        pop_values(&mut state, 2, instr.offset, "binary operands", self.mode)?;
        state.values.push(StackValue::Unknown);
      }
    }

    Ok(Some((state, Flow::Next)))
  }
}

fn enqueue_state(idx: usize, incoming: StackState, run: &mut FunctionRun, mode: &ProblemMode) -> Result<()> {
  match run.states.get_mut(&idx) {
    Some(existing) => {
      if merge_state(existing, &incoming, mode)? {
        run.worklist.push_back(idx);
      }
    }
    None => {
      run.states.insert(idx, incoming);
      run.worklist.push_back(idx);
    }
  }
  Ok(())
}

fn merge_state(existing: &mut StackState, incoming: &StackState, mode: &ProblemMode) -> Result<bool> {
  if existing.values.len() != incoming.values.len() {
    handle_problem(
      mode,
      format!(
        "inconsistent stack height at control-flow join: {} vs {}",
        existing.values.len(),
        incoming.values.len()
      ),
    )?;
    return Ok(false);
  }

  let mut changed = false;
  for (old, new) in existing.values.iter_mut().zip(incoming.values.iter()) {
    let merged = merge_value(old, new);
    if *old != merged {
      *old = merged;
      changed = true;
    }
  }
  Ok(changed)
}

fn merge_value(a: &StackValue, b: &StackValue) -> StackValue {
  if a == b {
    a.clone()
  } else {
    StackValue::Unknown
  }
}

fn syscall_arg_count(instr: &Instruction, syscalls: &[Syscall], mode: &ProblemMode) -> Result<usize> {
  let Operand::U16(index) = instr.operand else {
    bail!("syscall at 0x{:06X} has invalid operand", instr.offset);
  };
  let Some(syscall) = syscalls.get(index as usize) else {
    handle_problem(
      mode,
      format!("syscall at 0x{:06X} uses invalid index {}", instr.offset, index),
    )?;
    return Ok(0);
  };
  let arg_count = syscall.args_count;
  if arg_count < 0 {
    handle_problem(
      mode,
      format!(
        "syscall at 0x{:06X} uses negative argument count {}",
        instr.offset, arg_count
      ),
    )?;
    return Ok(0);
  }
  Ok(arg_count as usize)
}

fn address_operand(instr: &Instruction) -> Result<u32> {
  match instr.operand {
    Operand::Address(addr) => Ok(addr),
    _ => bail!(
      "{} at 0x{:06X} has invalid address operand",
      opcode_mnemonic(instr.opcode),
      instr.offset
    ),
  }
}

fn int_operand(instr: &Instruction) -> Result<i32> {
  match instr.operand {
    Operand::Int(value, _) => Ok(value),
    _ => bail!("push_int at 0x{:06X} has invalid integer operand", instr.offset),
  }
}

fn replace_top_unknown(state: &mut StackState, offset: u32, name: &str, mode: &ProblemMode) -> Result<()> {
  let Some(value) = state.values.last_mut() else {
    handle_problem(mode, format!("{} at 0x{:06X} reads an empty stack", name, offset))?;
    return Ok(());
  };
  *value = StackValue::Unknown;
  Ok(())
}

fn pop_values(state: &mut StackState, count: usize, offset: u32, name: &str, mode: &ProblemMode) -> Result<()> {
  if state.values.len() < count {
    handle_problem(
      mode,
      format!(
        "{} at 0x{:06X} needs {} value(s), but stack has {}",
        name,
        offset,
        count,
        state.values.len()
      ),
    )?;
    state.values.clear();
    return Ok(());
  }
  state.values.truncate(state.values.len() - count);
  Ok(())
}

fn handle_problem(mode: &ProblemMode, message: String) -> Result<()> {
  match mode {
    ProblemMode::Strict => bail!("{}", message),
    ProblemMode::Warn => {
      eprintln!("warning: {}", message);
      Ok(())
    }
  }
}
