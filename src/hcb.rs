use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::io::Cursor;
use std::path::Path;

use anyhow::{bail, Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};

use crate::utils::fs::{create_dir, read_file, read_file_to_string, write_file};
use crate::utils::io::*;
use crate::utils::num::parse_int;
use crate::utils::opcode::*;
use crate::utils::text::{decode_sjis, encode_sjis};

#[derive(Serialize, Deserialize)]
struct ConfigSyscall {
  id: usize,
  name: String,
  args_count: u8,
}

#[derive(Serialize, Deserialize)]
struct ConfigCustomSyscall {
  id: usize,
  name: String,
  args_count: u8,
  address: String,
}

#[derive(Serialize, Deserialize)]
struct Config {
  entry_point: String,
  non_volatile_global_count: u16,
  volatile_global_count: u16,
  game_mode: u8,
  game_title: String,
  syscalls: Vec<ConfigSyscall>,
  custom_syscalls: Vec<ConfigCustomSyscall>,
}

struct Syscall {
  name: String,
  args_count: i8,
}

struct CustomSyscall {
  name: String,
  args_count: i8,
  address: u32,
}

struct Descriptor {
  entry_point: u32,
  volatile_global_count: i16,
  non_volatile_global_count: i16,
  game_mode: i8,
  title: String,
  syscalls: Vec<Syscall>,
  custom_syscalls: Vec<CustomSyscall>,
}

fn parse_descriptor(data: &[u8]) -> Result<Descriptor> {
  let mut cur = Cursor::new(data);

  let entry_point = read_u32_le(&mut cur)?;
  let volatile_global_count = read_i16_le(&mut cur)?;
  let non_volatile_global_count = read_i16_le(&mut cur)?;
  let game_mode = read_i8(&mut cur)?;
  let _ = read_u8(&mut cur)?;

  let title_len = read_u8(&mut cur)? as usize;
  let title = decode_sjis(&read_bytes(&mut cur, title_len)?);

  let syscall_count = read_i16_le(&mut cur)? as usize;
  let mut syscalls = Vec::with_capacity(syscall_count);
  for _ in 0..syscall_count {
    let args_count = read_i8(&mut cur)?;
    let name_len = read_u8(&mut cur)? as usize;
    let name = decode_sjis(&read_bytes(&mut cur, name_len)?);
    syscalls.push(Syscall { name, args_count });
  }

  let custom_syscall_count = read_i16_le(&mut cur)? as usize;
  let mut custom_syscalls = Vec::with_capacity(custom_syscall_count);
  for _ in 0..custom_syscall_count {
    let address = read_u32_le(&mut cur)?;
    let args_count = read_i8(&mut cur)?;
    let name_len = read_u8(&mut cur)? as usize;
    let name = decode_sjis(&read_bytes(&mut cur, name_len)?);
    custom_syscalls.push(CustomSyscall {
      name,
      args_count,
      address,
    });
  }

  Ok(Descriptor {
    entry_point,
    volatile_global_count,
    non_volatile_global_count,
    game_mode,
    title,
    syscalls,
    custom_syscalls,
  })
}

fn write_descriptor(desc: &Descriptor) -> Result<Vec<u8>> {
  let mut buf = Vec::new();

  buf.extend_from_slice(&desc.entry_point.to_le_bytes());
  buf.extend_from_slice(&desc.volatile_global_count.to_le_bytes());
  buf.extend_from_slice(&desc.non_volatile_global_count.to_le_bytes());
  buf.push(desc.game_mode as u8);
  buf.push(0u8); // reserved

  let title_sjis = encode_sjis(&desc.title)?;
  if title_sjis.len() > 255 {
    bail!("game title exceeds 255 bytes");
  }
  buf.push(title_sjis.len() as u8);
  buf.extend_from_slice(&title_sjis);

  if desc.syscalls.len() > i16::MAX as usize {
    bail!("too many syscalls: {} > {}", desc.syscalls.len(), i16::MAX);
  }
  buf.extend_from_slice(&(desc.syscalls.len() as i16).to_le_bytes());
  for sc in &desc.syscalls {
    buf.push(sc.args_count as u8);
    let name_sjis = encode_sjis(&sc.name)?;
    if name_sjis.len() > 255 {
      bail!("syscall name '{}' exceeds 255 bytes", sc.name);
    }
    buf.push(name_sjis.len() as u8);
    buf.extend_from_slice(&name_sjis);
  }

  if desc.custom_syscalls.len() > i16::MAX as usize {
    bail!(
      "too many custom syscalls: {} > {}",
      desc.custom_syscalls.len(),
      i16::MAX
    );
  }
  buf.extend_from_slice(&(desc.custom_syscalls.len() as i16).to_le_bytes());
  for cs in &desc.custom_syscalls {
    buf.extend_from_slice(&cs.address.to_le_bytes());
    buf.push(cs.args_count as u8);
    let name_sjis = encode_sjis(&cs.name)?;
    if name_sjis.len() > 255 {
      bail!("custom syscall name '{}' exceeds 255 bytes", cs.name);
    }
    buf.push(name_sjis.len() as u8);
    buf.extend_from_slice(&name_sjis);
  }

  Ok(buf)
}

struct CodeTargets {
  functions: BTreeSet<u32>,
  jump_targets: BTreeSet<u32>,
  thread_targets: BTreeMap<u32, u32>,
}

fn collect_code_targets(mut cur: Cursor<&[u8]>, desc: &Descriptor) -> Result<CodeTargets> {
  let mut functions = BTreeSet::new();
  let mut jump_targets = BTreeSet::new();
  let mut thread_targets = BTreeMap::new();

  let mut thread_candidate: Option<(u32, u32)> = None;

  let ts_idx = desc
    .syscalls
    .iter()
    .position(|s| s.name == "ThreadStart")
    .map(|i| i as u16);

  while (cur.position() as usize) < cur.get_ref().len() {
    let off = cur.position() as u32;
    let op = decode_opcode(read_u8(&mut cur)?).with_context(|| format!("at offset 0x{:06X}", off))?;

    match op {
      Opcode::InitStack => {
        functions.insert(off);
      }
      Opcode::Jmp | Opcode::Jz => {
        jump_targets.insert(read_u32_le(&mut cur)?);
        thread_candidate = None;
        continue;
      }
      Opcode::PushIntI32 => {
        thread_candidate = Some((off, read_u32_le(&mut cur)?));
        continue;
      }
      Opcode::SysCall => {
        if Some(read_u16_le(&mut cur)?) != ts_idx {
          continue;
        }

        match thread_candidate {
          Some((push_off, val)) => {
            if functions.contains(&val) {
              thread_targets.insert(push_off, val);
            } else {
              eprintln!(
                "warning: ThreadStart at 0x{:06X} references 0x{:06X} which is not at beginning of function",
                off, val
              );
            }
          }
          _ => eprintln!(
            "warning: orphaned ThreadStart at 0x{:06X} is not preceded by push_int",
            off
          ),
        }

        thread_candidate = None;
        continue;
      }
      Opcode::PushString => {
        let len = read_u8(&mut cur)? as u64;
        cur.set_position(cur.position() + len);
        thread_candidate = None;
        continue;
      }
      _ => {}
    }
    cur.set_position(cur.position() + (opcode_size(op) - 1) as u64);
    thread_candidate = None;
  }

  Ok(CodeTargets {
    functions,
    jump_targets,
    thread_targets,
  })
}

pub fn disasm(input: &Path, output_dir: &Path) -> Result<()> {
  let data = read_file(input)?;
  if data.len() < size_of::<u32>() {
    bail!("file too small");
  }

  let descriptor_offset = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
  if descriptor_offset > data.len() {
    bail!(
      "descriptor offset 0x{:X} exceeds file size 0x{:X}",
      descriptor_offset,
      data.len()
    );
  }

  let desc = parse_descriptor(&data[descriptor_offset..])?;

  let mut cur = Cursor::new(&data[..descriptor_offset]);
  cur.set_position(size_of::<u32>() as u64);

  let targets = collect_code_targets(cur.clone(), &desc)?;

  let code_size = cur.get_ref().len() - cur.position() as usize;
  println!("{} {}", "Title:".bold(), desc.title);
  println!("{} 0x{:06X}", "Entry point:".bold(), desc.entry_point);
  println!("{} {} bytes", "Code size:".bold(), code_size);
  println!("{} {}", "Syscalls:".bold(), desc.syscalls.len());
  println!("{} {}", "Custom syscalls:".bold(), desc.custom_syscalls.len());
  println!("{} {}", "Functions:".bold(), targets.functions.len());

  let mut asm = String::new();
  let mut strings: Vec<String> = Vec::new();

  while (cur.position() as usize) < cur.get_ref().len() {
    let off = cur.position() as u32;
    let byte = read_u8(&mut cur)?;
    let op = decode_opcode(byte).with_context(|| format!("at offset 0x{:06X}", off))?;

    if targets.functions.contains(&off) {
      asm.push_str("\n; ======================================================\n\n");
      asm.push_str(&format!("sub_{:06X}:\n", off));
    }

    if targets.jump_targets.contains(&off) && !targets.functions.contains(&off) {
      asm.push_str(&format!("loc_{:06X}:\n", off));
    }

    let mnemonic = opcode_mnemonic(op);
    match op {
      Opcode::InitStack => {
        let arg_count = read_u8(&mut cur)?;
        let local_count = read_u8(&mut cur)?;
        asm.push_str(&format!("\t{} {} {}\n", mnemonic, arg_count, local_count));
      }
      Opcode::Call => {
        let addr = read_u32_le(&mut cur)?;
        if targets.functions.contains(&addr) {
          asm.push_str(&format!("\t{} LABEL:sub_{:06X}\n", mnemonic, addr));
        } else {
          eprintln!(
            "warning: call at 0x{:06X} targets 0x{:06X} which is not at beginning of function",
            off, addr
          );
          asm.push_str(&format!("\t{} 0x{:06X}\n", mnemonic, addr));
        }
      }
      Opcode::SysCall => {
        let idx = read_u16_le(&mut cur)?;
        if (idx as usize) < desc.syscalls.len() {
          asm.push_str(&format!("\t{} {}\n", mnemonic, desc.syscalls[idx as usize].name));
        } else {
          asm.push_str(&format!("\t{} {}\n", mnemonic, idx));
        }
      }
      Opcode::Jmp | Opcode::Jz => {
        let addr = read_u32_le(&mut cur)?;
        let current_func = targets.functions.range(..=off).next_back();
        let target_func = targets.functions.range(..=addr).next_back();
        if targets.functions.contains(&addr) || current_func != target_func {
          eprintln!(
            "warning: {} at 0x{:06X} targets 0x{:06X} which is beyond boundary of current function",
            mnemonic, off, addr
          );
        }
        if targets.functions.contains(&addr) {
          asm.push_str(&format!("\t{} LABEL:sub_{:06X}\n", mnemonic, addr));
        } else {
          asm.push_str(&format!("\t{} LABEL:loc_{:06X}\n", mnemonic, addr));
        }
      }
      Opcode::PushIntI32 => {
        let val = read_i32_le(&mut cur)?;
        if let Some(&target) = targets.thread_targets.get(&off) {
          asm.push_str(&format!("\t{} LABEL:sub_{:06X}\n", mnemonic, target));
        } else {
          asm.push_str(&format!("\t{} {}\n", mnemonic, val));
        }
      }
      Opcode::PushIntI16 => {
        let val = read_i16_le(&mut cur)?;
        asm.push_str(&format!("\t{} {}\n", mnemonic, val as i32));
      }
      Opcode::PushIntI8 => {
        let val = read_i8(&mut cur)?;
        asm.push_str(&format!("\t{} {}\n", mnemonic, val as i32));
      }
      Opcode::PushFloat => {
        let val = read_f32_le(&mut cur)?;
        asm.push_str(&format!("\t{} {:?}\n", mnemonic, val));
      }
      Opcode::PushString => {
        let len = read_u8(&mut cur)? as usize;
        let pos = cur.position() as usize;
        if pos + len > cur.get_ref().len() {
          bail!("push_string data extends past code segment");
        }
        cur.set_position((pos + len) as u64);
        strings.push(decode_sjis(&cur.get_ref()[pos..pos + len]));
        asm.push_str(&format!("\t{} STRING:{}\n", mnemonic, strings.len()));
      }
      Opcode::PushGlbvar | Opcode::PushGlbvarTable | Opcode::PopGlbvar | Opcode::PopGlbvarTable => {
        let idx = read_u16_le(&mut cur)?;
        asm.push_str(&format!("\t{} {}\n", mnemonic, idx));
      }
      Opcode::PushStkvar | Opcode::PushStkvarTable | Opcode::PopStkvar | Opcode::PopStkvarTable => {
        let idx = read_i8(&mut cur)?;
        asm.push_str(&format!("\t{} {}\n", mnemonic, idx));
      }
      _ => {
        asm.push_str(&format!("\t{}\n", mnemonic));
      }
    }
  }

  create_dir(output_dir)?;

  let dir_name = output_dir
    .file_name()
    .map(|n| n.to_string_lossy().into_owned())
    .unwrap_or_else(|| "output".to_string());
  write_file(&output_dir.join(format!("{}.asm", dir_name)), asm.as_bytes())?;

  let strings_path = output_dir.join("strings.txt");
  crate::utils::strings::save(&strings_path, &strings)?;

  let config_path = output_dir.join("config.yaml");
  let config = Config {
    entry_point: format!("sub_{:06X}", desc.entry_point),
    non_volatile_global_count: desc.non_volatile_global_count as u16,
    volatile_global_count: desc.volatile_global_count as u16,
    game_mode: desc.game_mode as u8,
    game_title: desc.title.clone(),
    syscalls: desc
      .syscalls
      .iter()
      .enumerate()
      .map(|(i, sc)| ConfigSyscall {
        id: i,
        name: sc.name.clone(),
        args_count: sc.args_count as u8,
      })
      .collect(),
    custom_syscalls: desc
      .custom_syscalls
      .iter()
      .enumerate()
      .map(|(i, cs)| {
        let address = if targets.functions.contains(&cs.address) {
          format!("sub_{:06X}", cs.address)
        } else {
          eprintln!(
            "warning: custom syscall '{}' targets 0x{:06X} which is not at beginning of function",
            cs.name, cs.address
          );
          format!("0x{:06X}", cs.address)
        };
        ConfigCustomSyscall {
          id: i,
          name: cs.name.clone(),
          args_count: cs.args_count as u8,
          address,
        }
      })
      .collect(),
  };
  let config_yaml =
    serde_yaml::to_string(&config).with_context(|| format!("serializing config for '{}'", config_path.display()))?;
  write_file(&config_path, config_yaml.as_bytes())?;

  Ok(())
}

fn resolve_label_or_address(s: &str, labels: &HashMap<String, u32>) -> Option<u32> {
  labels.get(s).copied().or_else(|| parse_int(s))
}

pub fn asm(input_dir: &Path, output: &Path) -> Result<()> {
  let config_path = input_dir.join("config.yaml");
  let config: Config = serde_yaml::from_str(&read_file_to_string(&config_path)?)
    .with_context(|| format!("parsing '{}'", config_path.display()))?;

  let strings_path = input_dir.join("strings.txt");
  let strings = if strings_path.exists() {
    crate::utils::strings::load(&strings_path)?
  } else {
    Vec::new()
  };

  let dir_name = input_dir
    .file_name()
    .map(|n| n.to_string_lossy().into_owned())
    .unwrap_or_else(|| "output".to_string());
  let asm_text = read_file_to_string(&input_dir.join(format!("{}.asm", dir_name)))?;

  let syscall_index: HashMap<&str, u16> = config
    .syscalls
    .iter()
    .enumerate()
    .map(|(i, sc)| (sc.name.as_str(), i as u16))
    .collect();

  let mut instructions: Vec<ParsedInstruction> = Vec::new();
  let mut instruction_offsets: Vec<u32> = Vec::new();
  let mut labels: HashMap<String, u32> = HashMap::new();
  let mut offset: u32 = size_of::<u32>() as u32;

  for (line_num, line) in asm_text.lines().enumerate() {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with(';') {
      continue;
    }

    if trimmed.ends_with(':') && !trimmed.contains(' ') {
      let label_name = &trimmed[..trimmed.len() - 1];
      if labels.insert(label_name.to_string(), offset).is_some() {
        bail!("line {}: duplicate label '{}'", line_num + 1, label_name);
      }
      continue;
    }

    let parts: Vec<&str> = trimmed.split(' ').collect();
    if parts.is_empty() {
      continue;
    }

    let mnemonic = parts[0];
    let op = mnemonic_to_opcode(mnemonic)
      .with_context(|| format!("line {}: unknown mnemonic '{}'", line_num + 1, mnemonic))?;

    let instr_offset = offset;
    let instr = match op {
      Opcode::InitStack => {
        let arg_count: u8 = parts[1]
          .parse()
          .with_context(|| format!("line {}: invalid arg_count on init_stack", line_num + 1))?;
        let local_count: u8 = parts[2]
          .parse()
          .with_context(|| format!("line {}: invalid local_count on init_stack", line_num + 1))?;
        offset += opcode_size(op);
        ParsedInstruction {
          opcode: op,
          operand: Operand::InitStack { arg_count, local_count },
        }
      }
      Opcode::Call => {
        let operand = parts[1];
        offset += opcode_size(op);
        if let Some(label) = operand.strip_prefix("LABEL:") {
          ParsedInstruction {
            opcode: op,
            operand: Operand::Label(label.to_string()),
          }
        } else {
          let addr: u32 =
            parse_int(operand).with_context(|| format!("line {}: invalid call target '{}'", line_num + 1, operand))?;
          ParsedInstruction {
            opcode: op,
            operand: Operand::Address(addr),
          }
        }
      }
      Opcode::Jmp | Opcode::Jz => {
        let operand = parts[1];
        offset += opcode_size(op);
        if let Some(label) = operand.strip_prefix("LABEL:") {
          ParsedInstruction {
            opcode: op,
            operand: Operand::Label(label.to_string()),
          }
        } else {
          let addr: u32 =
            parse_int(operand).with_context(|| format!("line {}: invalid jump target '{}'", line_num + 1, operand))?;
          ParsedInstruction {
            opcode: op,
            operand: Operand::Address(addr),
          }
        }
      }
      Opcode::SysCall => {
        let name = parts[1];
        if !syscall_index.contains_key(name) {
          bail!("line {}: unknown syscall '{}'", line_num + 1, name);
        }
        offset += opcode_size(op);
        ParsedInstruction {
          opcode: op,
          operand: Operand::SyscallName(name.to_string()),
        }
      }
      Opcode::PushIntI32 => {
        let operand = parts[1];
        if let Some(label) = operand.strip_prefix("LABEL:") {
          offset += opcode_size(Opcode::PushIntI32);
          ParsedInstruction {
            opcode: Opcode::PushIntI32,
            operand: Operand::Label(label.to_string()),
          }
        } else {
          let val: i32 =
            parse_int(operand).with_context(|| format!("line {}: invalid integer {}", line_num + 1, operand))?;
          let actual_op = if val >= i8::MIN as i32 && val <= i8::MAX as i32 {
            Opcode::PushIntI8
          } else if val >= i16::MIN as i32 && val <= i16::MAX as i32 {
            Opcode::PushIntI16
          } else {
            Opcode::PushIntI32
          };
          let encoding = match actual_op {
            Opcode::PushIntI8 => IntEncoding::I8,
            Opcode::PushIntI16 => IntEncoding::I16,
            _ => IntEncoding::I32,
          };
          offset += opcode_size(actual_op);
          ParsedInstruction {
            opcode: actual_op,
            operand: Operand::Int(val, encoding),
          }
        }
      }
      Opcode::PushFloat => {
        let val: f32 = parts[1]
          .parse()
          .with_context(|| format!("line {}: invalid float {}", line_num + 1, parts[1]))?;
        offset += opcode_size(op);
        ParsedInstruction {
          opcode: op,
          operand: Operand::Float(val),
        }
      }
      Opcode::PushString => {
        let raw = parts[1..].join(" ");
        let text = if let Some(idx_str) = raw.strip_prefix("STRING:") {
          let idx: usize = idx_str
            .parse::<usize>()
            .with_context(|| format!("line {}: invalid string index", line_num + 1))?;
          if idx == 0 || idx > strings.len() {
            bail!(
              "line {}: string reference {} out of range (have {} strings)",
              line_num + 1,
              idx,
              strings.len()
            );
          }
          &strings[idx - 1]
        } else {
          &raw
        };
        let sjis = encode_sjis(text)?;
        offset += opcode_size(op) + sjis.len() as u32;
        ParsedInstruction {
          opcode: op,
          operand: Operand::String(sjis),
        }
      }
      Opcode::PushGlbvar | Opcode::PushGlbvarTable | Opcode::PopGlbvar | Opcode::PopGlbvarTable => {
        let val: u16 = parts[1]
          .parse()
          .with_context(|| format!("line {}: invalid u16 operand", line_num + 1))?;
        offset += opcode_size(op);
        ParsedInstruction {
          opcode: op,
          operand: Operand::U16(val),
        }
      }
      Opcode::PushStkvar | Opcode::PushStkvarTable | Opcode::PopStkvar | Opcode::PopStkvarTable => {
        let val: i8 = parts[1]
          .parse()
          .with_context(|| format!("line {}: invalid i8 operand", line_num + 1))?;
        offset += opcode_size(op);
        ParsedInstruction {
          opcode: op,
          operand: Operand::I8(val),
        }
      }
      _ => {
        offset += opcode_size(op);
        ParsedInstruction {
          opcode: op,
          operand: Operand::None,
        }
      }
    };

    instruction_offsets.push(instr_offset);
    instructions.push(instr);
  }

  let function_starts: BTreeSet<u32> = instructions
    .iter()
    .zip(instruction_offsets.iter())
    .filter(|(instr, _)| matches!(instr.opcode, Opcode::InitStack))
    .map(|(_, &off)| off)
    .collect();

  let ts_idx = config.syscalls.iter().position(|s| s.name == "ThreadStart");

  for (i, instr) in instructions.iter().enumerate() {
    let off = instruction_offsets[i];
    match (instr.opcode, &instr.operand) {
      (Opcode::Call, Operand::Label(name)) => {
        let target = *labels
          .get(name)
          .with_context(|| format!("undefined label: '{}'", name))?;
        if !function_starts.contains(&target) {
          bail!(
            "call target '{}' (0x{:06X}) does not point at beginning of function",
            name,
            target
          );
        }
      }
      (Opcode::Jmp | Opcode::Jz, Operand::Label(name)) => {
        let target = *labels
          .get(name)
          .with_context(|| format!("undefined label: '{}'", name))?;
        let jmp_func = function_starts.range(..=off).next_back();
        let target_func = function_starts.range(..=target).next_back();
        if function_starts.contains(&target) || jmp_func != target_func {
          bail!(
            "{} target '{}' (0x{:06X}) points at outside boundary of current function",
            opcode_mnemonic(instr.opcode),
            name,
            target
          );
        }
      }
      _ => {}
    }

    if matches!(instr.opcode, Opcode::SysCall) && ts_idx.is_some() {
      if let Operand::SyscallName(name) = &instr.operand {
        if name == "ThreadStart" {
          let mut j = i;
          while j > 0 {
            j -= 1;
            match (instructions[j].opcode, &instructions[j].operand) {
              (Opcode::PushIntI32, Operand::Label(label)) => {
                let target = *labels
                  .get(label)
                  .with_context(|| format!("ThreadStart references unknown label '{}'", label))?;
                if !function_starts.contains(&target) {
                  bail!(
                    "ThreadStart label '{}' (0x{:06X}) does not point at beginning of function",
                    label,
                    target
                  );
                }
                break;
              }
              (Opcode::PushIntI32 | Opcode::PushIntI8 | Opcode::PushIntI16, _) => {}
              _ => break,
            }
          }
        }
      }
    }
  }

  let mut code = Vec::with_capacity(offset as usize);

  for instr in &instructions {
    code.push(encode_opcode(instr.opcode));
    match &instr.operand {
      Operand::None => {}
      Operand::InitStack { arg_count, local_count } => {
        code.push(*arg_count);
        code.push(*local_count);
      }
      Operand::Label(name) => {
        let addr = labels
          .get(name)
          .with_context(|| format!("undefined label: '{}'", name))?;
        code.extend_from_slice(&addr.to_le_bytes());
      }
      Operand::Address(addr) => {
        code.extend_from_slice(&addr.to_le_bytes());
      }
      Operand::SyscallName(name) => {
        let idx = syscall_index[name.as_str()];
        code.extend_from_slice(&idx.to_le_bytes());
      }
      Operand::String(sjis) => {
        if sjis.len() > u8::MAX as usize {
          bail!(
            "push_string: sjis payload of {} bytes exceeds max of {}",
            sjis.len(),
            u8::MAX
          );
        }
        code.push(sjis.len() as u8);
        code.extend_from_slice(sjis);
      }
      Operand::Int(val, encoding) => match encoding {
        IntEncoding::I8 => {
          code.push(*val as u8);
        }
        IntEncoding::I16 => {
          code.extend_from_slice(&(*val as i16).to_le_bytes());
        }
        IntEncoding::I32 => {
          code.extend_from_slice(&val.to_le_bytes());
        }
      },
      Operand::Float(val) => {
        code.extend_from_slice(&val.to_le_bytes());
      }
      Operand::U16(val) => {
        code.extend_from_slice(&val.to_le_bytes());
      }
      Operand::I8(val) => {
        code.push(*val as u8);
      }
    }
  }

  eprintln!("assembled {} bytes of code", code.len());

  let entry_point = resolve_label_or_address(&config.entry_point, &labels)
    .with_context(|| format!("entry point '{}' is not a valid label or address", config.entry_point))?;

  let mut custom_syscalls = Vec::with_capacity(config.custom_syscalls.len());
  for cs in &config.custom_syscalls {
    let address = resolve_label_or_address(&cs.address, &labels).with_context(|| {
      format!(
        "custom syscall address '{}' is not a valid label or address",
        cs.address
      )
    })?;
    custom_syscalls.push(CustomSyscall {
      name: cs.name.clone(),
      args_count: cs.args_count as i8,
      address,
    });
  }

  if config.volatile_global_count > i16::MAX as u16 {
    bail!(
      "too many volatile global variables: {} > {}",
      config.volatile_global_count,
      i16::MAX
    );
  }
  if config.non_volatile_global_count > i16::MAX as u16 {
    bail!(
      "too many non-volatile global variables: {} > {}",
      config.non_volatile_global_count,
      i16::MAX
    );
  }

  let desc = Descriptor {
    entry_point,
    volatile_global_count: config.volatile_global_count as i16,
    non_volatile_global_count: config.non_volatile_global_count as i16,
    game_mode: config.game_mode as i8,
    title: config.game_title,
    syscalls: config
      .syscalls
      .into_iter()
      .map(|sc| Syscall {
        name: sc.name,
        args_count: sc.args_count as i8,
      })
      .collect(),
    custom_syscalls,
  };
  let descriptor_bytes = write_descriptor(&desc)?;
  let descriptor_offset = size_of::<u32>() + code.len();

  let mut out_data = Vec::with_capacity(descriptor_offset + descriptor_bytes.len());
  out_data.extend_from_slice(&(descriptor_offset as u32).to_le_bytes());
  out_data.extend_from_slice(&code);
  out_data.extend_from_slice(&descriptor_bytes);

  write_file(output, &out_data)?;
  eprintln!("wrote '{}' ({} bytes)", output.display(), out_data.len());

  Ok(())
}
