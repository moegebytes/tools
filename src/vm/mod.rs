pub mod instruction;
pub mod opcode;
pub mod stack;

pub(crate) struct Syscall {
  pub name: String,
  pub args_count: i8,
}

pub(crate) struct CustomSyscall {
  pub name: String,
  pub args_count: i8,
  pub address: u32,
}
