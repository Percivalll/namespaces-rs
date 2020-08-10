use super::basic;
use anyhow::anyhow;
use ipc_channel::ipc;
use libc;
use nix::unistd;
use serde::{Deserialize, Serialize};
pub fn setns_spawn<F, T>(n: basic::Namespace, f: F) -> anyhow::Result<T>
where
    F: FnOnce() -> T,
    F: Send + 'static,
    T: Send + for<'de> Deserialize<'de> + Serialize + 'static + std::fmt::Debug,
{
    let (tx, rx) = ipc::channel().unwrap();
    match unistd::fork() {
        Ok(unistd::ForkResult::Parent { child, .. }) => {
            let res = match rx.recv() {
                Ok(res) => res,
                Err(e) => return Err(anyhow!("{:?}", e)),
            };
            return Ok(res);
        }
        Ok(unistd::ForkResult::Child) => unsafe {
            if libc::setns(n.get_fd().unwrap(), 0) == -1 {
                println!("{:?}", std::io::Error::last_os_error());
            }
            tx.send(f());
            libc::exit(0)
        },
        Err(e) => Err(anyhow!("{}", e)),
    }
}
pub fn setns_spawn_all<F, T>(
    ns: basic::Namespaces,
    f: F,
) -> anyhow::Result<std::collections::HashMap<String, T>>
where
    F: Fn() -> T,
    F: Send + 'static,
    T: Send + for<'de> Deserialize<'de> + Serialize + 'static + std::fmt::Debug,
{
    let (tx, rx) = ipc::channel().unwrap();
    match unistd::fork() {
        Ok(unistd::ForkResult::Parent { child, .. }) => {
            let res = match rx.recv() {
                Ok(res) => res,
                Err(e) => return Err(anyhow!("{:?}", e)),
            };
            return Ok(res);
        }
        Ok(unistd::ForkResult::Child) => unsafe {
            let mut res = std::collections::HashMap::new();
            let ori_fd = ns.get("origin").unwrap().get_fd().unwrap();
            for (k, v) in ns {
                let fd = match v.get_fd() {
                    Some(fd) => fd,
                    None => continue,
                };
                if libc::setns(fd, 0) == -1 {
                    println!("{:?}", std::io::Error::last_os_error());
                }
                res.insert(k, f());
                if libc::close(fd) == -1 {
                    println!("{:?}", std::io::Error::last_os_error());
                };
                if libc::setns(ori_fd, 0) == -1 {
                    println!("{:?}", std::io::Error::last_os_error());
                };
            }
            tx.send(res);
            libc::close(ori_fd);
            libc::exit(0)
        },
        Err(e) => Err(anyhow!("{}", e)),
    }
}
