use libc::termios as Termios;
use libc::{c_int, ioctl, TIOCGWINSZ};
use std::io::{BufWriter, Read, Write};
use std::os::fd::{AsFd, AsRawFd, RawFd};
use std::{fs, io, mem};
use std::fmt;

#[repr(C)]
struct WinSize {
    row: u16,
    col: u16,
    x: u16,
    y: u16,
}

struct Terminal {
    prev_ios: Termios,
    fd: RawFd,
    output: io::BufWriter<fs::File>,
    size: WinSize,
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.flush();
        let _ = Self::set_attr(self.fd, &self.prev_ios);
    }
}

impl Write for Terminal {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.output.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

impl Terminal {
    fn check_errno<Errno: PartialEq<c_int>>(errno: Errno) -> io::Result<Errno> {
        if errno == -1 {
            Err(io::Error::last_os_error())
        } else {
            Ok(errno)
        }
    }

    fn make_raw(termios: &mut Termios) {
        unsafe { libc::cfmakeraw(termios) }
    }

    fn get_attr(fd: RawFd) -> io::Result<Termios> {
        unsafe {
            let mut termios = mem::zeroed();
            Self::check_errno(libc::tcgetattr(fd, &mut termios))?;
            Ok(termios)
        }
    }

    fn set_attr(fd: RawFd, termios: &Termios) -> io::Result<()> {
        let err = unsafe { libc::tcsetattr(fd, libc::TCSANOW, termios) };
        Self::check_errno(err).and(Ok(()))
    }

    fn get_window_size(fd: RawFd) -> io::Result<WinSize> {
        unsafe {
            let mut size: WinSize = mem::zeroed();
            Self::check_errno(ioctl(fd, TIOCGWINSZ.into(), &mut size as *mut _))?;
            Ok(size)
        }
    }

    fn get_tty() -> io::Result<Terminal> {
        let f = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/tty")?;

        let mut term = Terminal {
            prev_ios: Self::get_attr(f.as_fd().as_raw_fd())?,
            fd: f.as_fd().as_raw_fd(),
            size: WinSize {
                row: 80,
                col: 24,
                x: 80,
                y: 24,
            },
            output: BufWriter::with_capacity(1024, f),
        };

        term.activate_raw_mode()?;
        term.size = Self::get_window_size(term.as_raw_fd())?;
        Ok(term)
    }

    fn activate_raw_mode(&self) -> io::Result<()> {
        let mut ios = Self::get_attr(self.as_raw_fd())?;
        Self::make_raw(&mut ios);
        Self::set_attr(self.as_raw_fd(), &ios)?;
        Ok(())
    }

    fn suspend_raw_mode(&self) -> io::Result<()> {
        Self::set_attr(self.as_raw_fd(), &self.prev_ios)
    }

    fn clear_screen(&mut self) {
        // clear屏幕
        let _ = self.write(b"\x1b[2J");
        // 定位到屏幕左上角
        let _ = self.write(b"\x1b[H");
    }

    fn move_cursor(&mut self, x: u16, y: u16) {
        // \x1b[65535;65535H
        // let mut buf = [0u8; 16];
        // let _ = self.write(b"\x1b[");
        // let _ = self.write(x.to_string().as_bytes());
        // write!(&mut buf, "\x1b[{};{}H", x, y);
        // let _ = self.write(buf);
    }

    fn draw_rows(&mut self) {
        for y in 0..self.size.row {
            if y == self.size.row / 3 {
                let buf = b"emv editor -- version 0.1";
                let start = (self.size.col - buf.len() as u16) / 2;
                self.write(b"~").unwrap();
                for _ in 1..start {
                    self.write(b" ").unwrap();
                }
                self.write(buf).unwrap();
            } else {
                let _ = self.write(b"~");
            }
            let _ = self.write(b"\x1b[K");
            if y < self.size.row - 1 {
                let _ = self.write(b"\r\n");
            }
        }
        let _ = self.write(b"\x1b[H");
    }

    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }

    fn refresh_screen(&mut self) -> io::Result<()> {
        // ignore cursor
        self.write(b"\x1b[?25l")?;
        let _ = self.write(b"\x1b[H");
        self.draw_rows();
        // show cursor
        self.write(b"\x1b[?25h")?;
        self.flush();
        Ok(())
    }

    fn flush(&mut self) {
        let _ = self.output.flush();
    }
}

pub fn main() -> io::Result<()> {
    let mut tty = Terminal::get_tty()?;
    let mut stdin = io::stdin();
    let mut buf = [0u8; 1];

    tty.clear_screen();
    tty.draw_rows();
    tty.flush();

    loop {
        match stdin.read_exact(&mut buf) {
            Ok(_) => {
                if buf[0] == b'q' {
                    tty.clear_screen();
                    break;
                }
                if buf[0] == b'r' {
                    tty.refresh_screen()?;
                    continue;
                }
                print!("buf {}\r\n\x1b[K", buf[0])
            }
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
