use nix::unistd::{Uid, Gid, Group};
use std::process::exit;
use std::fs;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    if !Uid::is_root(Uid::effective()) {
        eprintln!("Effective UID must be root");
        exit(1);
    }

    let group = Group::from_gid(Gid::effective()).unwrap().unwrap();

    if group.name == "root" {
        eprintln!("Effective GID must NOT be root");
        exit(1);
    }

    let mut fp = fs::File::create("/etc/sudoers.d/".to_string() + &group.name)?;

    writeln!(&mut fp, "%{} ALL=NOPASSWD: ALL", group.name)?;

    Ok(())
}
