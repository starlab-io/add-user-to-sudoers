use nix::libc::{gid_t, uid_t};
use nix::unistd::{Gid, Group, Uid, User};

use std::fs;
use std::io::{self, Write};
use std::process::exit;

const NEW_USER: &str = "docker_user";
const NEW_GROUP: &str = "docker_user";

/*
 * Adds the current uid/gid to /etc/passwd and /etc/shadow
 */
fn add_docker_user(uid: uid_t, gid: gid_t) -> io::Result<()> {
    //////////////////////
    // Update /etc/passwd
    //////////////////////
    let mut fp = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("/etc/passwd")?;

    let shell = "/bin/bash";
    let home = "/";
    let login = NEW_USER;
    let name = "Docker User";

    writeln!(
        &mut fp,
        "{}:x:{}:{}:{},,,:{}:{}",
        login, uid, gid, name, home, shell
    )?;

    //////////////////////
    // Update /etc/shadow
    //////////////////////

    let mut fp = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("/etc/shadow")?;

    writeln!(&mut fp, "{}:*:::::::", login)?;

    //////////////////////
    // Update /etc/group
    //////////////////////
    let mut fp = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open("/etc/group")?;

    let group = NEW_GROUP;

    writeln!(&mut fp, "{}:x:{}:", group, gid)?;
    Ok(())
}

fn create_sudoers_file() -> io::Result<()> {
    let mut fp = fs::File::create("/etc/sudoers.d/".to_string() + NEW_GROUP)?;
    writeln!(&mut fp, "%{} ALL=NOPASSWD: ALL", NEW_GROUP)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let uid: uid_t = uid_t::from(Uid::current());
    let gid: gid_t = gid_t::from(Gid::current());

    if !Uid::effective().is_root() {
        eprintln!("Effective UID must be root");
        exit(1);
    }

    if Uid::current().is_root() {
        eprintln!("Current UID must not be root");
        exit(1);
    }

    if gid == 0 {
        eprintln!("Current GID must NOT be root");
        exit(1);
    }

    // Make sure we don't already exist
    let user: Option<User> = User::from_uid(Uid::from_raw(uid)).unwrap();
    let group: Option<Group> = Group::from_gid(Gid::from_raw(gid)).unwrap();

    if user.is_some() {
        eprintln!("UID {} is already owned by {}!\n", uid, user.unwrap().name);
        exit(1);
    }

    if group.is_some() {
        eprintln!("GID {} is already owned by {}!\n", gid, group.unwrap().name);
        exit(1);
    }

    add_docker_user(uid, gid)?;
    create_sudoers_file()?;
    Ok(())
}
