use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum Opcode {
  DeviceList,
  Attach,
  AppVersion,
  Name,
  Close,
  Info,
  Boot,
  Menu,
  Reset,
  Binary,
  Stream,
  Fence,
  GetAddress,
  PutAddress,
  PutIPS,
  GetFile,
  PutFile,
  List,
  Remove,
  Rename,
  MakeDir,
}

impl Default for Opcode {
  fn default() -> Self { Opcode::DeviceList }
}

#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Space {
  Snes,
  Cmd,
}

impl Default for Space {
  fn default() -> Self { Space::Snes }
}

#[derive(Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct Request {
  opcode: Opcode,
  space: Space,
  #[serde(skip_serializing_if = "Option::is_none")]
  flags: Option<Vec<String>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  operands: Option<Vec<String>>,
}

impl Request {
  #[must_use]
  pub fn device_list() -> Self {
    Request {
      opcode: Opcode::DeviceList,
      space: Space::Snes,
      ..Self::default()
    }
  }

  #[must_use]
  pub fn attach(device: &str) -> Self {
    Request {
      opcode: Opcode::Attach,
      space: Space::Snes,
      operands: Some(vec![device.into()]),
      ..Self::default()
    }
  }

  #[must_use]
  pub fn info() -> Self {
    Request {
      opcode: Opcode::Info,
      space: Space::Snes,
      ..Self::default()
    }
  }

  #[must_use]
  pub fn get_address(offset: usize, length: usize) -> Self {
    Request {
      opcode: Opcode::GetAddress,
      space: Space::Snes,
      operands: Some(vec![
        format!("{:X}", offset),
        format!("{:X}", length),
      ]),
      ..Self::default()
    }
  }
}
