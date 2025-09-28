use crate::error::AppError;
use std::os::fd::AsRawFd;

pub fn read_secure(prompt: &str) -> Result<zeroize::Zeroizing<Vec<u8>>, AppError> {
  use std::fs::OpenOptions;
  use std::io::Read;

  let mut tty_r = OpenOptions::new()
    .read(true)
    .open("/dev/tty")
    .map_err(|e| AppError::Terminal(format!("open tty for read: {}", e)))?;

  println!("{}", prompt);

  let fd = tty_r.as_raw_fd();
  unsafe {
    let mut term: libc::termios = core::mem::zeroed();
    if libc::tcgetattr(fd, &mut term) != 0 {
      return Err(AppError::Terminal("tcgetattr failed".into()));
    }
    let mut noecho = term;
    noecho.c_lflag &= !libc::ECHO;
    if libc::tcsetattr(fd, libc::TCSANOW, &noecho) != 0 {
      return Err(AppError::Terminal("tcsetattr disable echo failed".into()));
    }

    let mut buf = zeroize::Zeroizing::new(Vec::with_capacity(64));
    let mut byte = [0u8; 1];
    loop {
      let n = tty_r
        .read(&mut byte)
        .map_err(|e| AppError::Terminal(format!("read: {}", e)))?;
      if n == 0 {
        break;
      }
      if byte[0] == b'\n' {
        break;
      }
      if byte[0] == b'\r' {
        continue;
      }
      buf.push(byte[0]);
    }

    if libc::tcsetattr(fd, libc::TCSANOW, &term) != 0 {
      return Err(AppError::Terminal("tcsetattr restore echo failed".into()));
    }

    while matches!(buf.last(), Some(b'\n' | b'\r')) {
      buf.pop();
    }
    Ok(buf)
  }
}

pub fn read_line(prompt: &str) -> Result<String, AppError> {
  use std::fs::OpenOptions;
  use std::io::Read;

  let mut tty_r = OpenOptions::new()
    .read(true)
    .open("/dev/tty")
    .map_err(|e| AppError::Terminal(format!("open tty for read: {}", e)))?;

  println!("{}", prompt);

  let mut s = String::new();
  loop {
    let mut byte = [0u8; 1];
    let n = tty_r
      .read(&mut byte)
      .map_err(|e| AppError::Terminal(format!("read: {}", e)))?;
    if n == 0 {
      break;
    }
    if byte[0] == b'\n' {
      break;
    }
    if byte[0] == b'\r' {
      continue;
    }
    s.push(byte[0] as char);
  }

  println!();
  Ok(s)
}
