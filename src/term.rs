use libc::c_int;
use libc::termios as Termios;
use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::{fs, io, mem};

struct RawTerminal<Fd: Read + Write + AsFd> {
    prev_ios: Termios,
    fd: Fd,
}

impl<Fd: Read + Write + AsFd> Drop for RawTerminal<Fd> {
    fn drop(&mut self) {
        let _ = Self::set_attr(self.fd.as_fd(), &self.prev_ios);
    }
}

impl<Fd: Read + Write + AsFd> AsFd for RawTerminal<Fd> {
    fn as_fd(&self) -> BorrowedFd {
        self.fd.as_fd()
    }
}

impl<Fd: Read + Write + AsFd> RawTerminal<Fd> {
    fn make_raw(termios: &mut Termios) {
        unsafe { libc::cfmakeraw(termios) }
    }

    fn get_attr(fd: BorrowedFd) -> io::Result<Termios> {
        unsafe {
            let mut termios = mem::zeroed();
            check_errno(libc::tcgetattr(fd.as_fd().as_raw_fd(), &mut termios))?;
            Ok(termios)
        }
    }

    fn set_attr(fd: BorrowedFd, termios: &Termios) -> io::Result<()> {
        let err = unsafe { libc::tcsetattr(fd.as_raw_fd(), libc::TCSANOW, termios) };
        check_errno(err).and(Ok(()))
    }

    fn suspend_raw_mode(&self) -> io::Result<()> {
        Self::set_attr(self.as_fd(), &self.prev_ios)
    }

    fn activate_raw_mode(&self) -> io::Result<()> {
        let mut ios = Self::get_attr(self.as_fd())?;
        Self::make_raw(&mut ios);
        Self::set_attr(self.as_fd(), &ios)?;
        Ok(())
    }

    fn fd(&self) -> &Fd {
        &self.fd
    }
}

// 给Fd添加进入raw模式的方法
trait IntoRawMode: Read + Write + AsFd + Sized {
    fn into_raw_mode(self) -> io::Result<RawTerminal<Self>>;
}

impl<Fd: Read + Write + AsFd> IntoRawMode for Fd {
    fn into_raw_mode(self) -> io::Result<RawTerminal<Fd>> {
        let term = RawTerminal {
            prev_ios: RawTerminal::<Fd>::get_attr(self.as_fd())?,
            fd: self,
        };
        term.activate_raw_mode()?;
        Ok(term)
    }
}

fn check_errno<Errno: PartialEq<c_int>>(errno: Errno) -> io::Result<Errno> {
    if errno == -1 {
        Err(io::Error::last_os_error())
    } else {
        Ok(errno)
    }
}

fn get_tty() -> io::Result<RawTerminal<fs::File>> {
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open("/dev/tty")?
        .into_raw_mode()
}

pub fn main() -> io::Result<()> {
    let tty = get_tty()?;
    let mut buffer = [0u8; 1];

    loop {
        match tty.fd().read_exact(&mut buffer) {
            Ok(_) => {
                if buffer[0] == b'q' {
                    break;
                }
                print!("buffer {}\r\n", buffer[0])
                // tty.fd().write_all(b"asdf").unwrap();
            },
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            // EAGAIN?
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
