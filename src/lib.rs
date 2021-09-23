#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod mac_os;

#[cfg(target_os = "macos")]
#[macro_use]
extern crate objc;

pub struct Sysbar(SysbarImpl);

impl Sysbar {
    pub fn new(name: &str) -> Self {
        Sysbar(SysbarImpl::new(name))
    }

    pub fn add_item(&mut self, label: &str, cbs: Box<dyn Fn()>) {
        self.0.add_item(label, cbs)
    }

    pub fn add_quit_item(&mut self, label: &str) {
        self.0.add_quit_item(label)
    }

    pub fn display(&mut self) {
        self.0.display()
    }
}

#[cfg(target_os = "macos")]
type SysbarImpl = mac_os::MacOsSysbar;

#[cfg(target_os = "linux")]
type SysbarImpl = linux::LinuxSysbar;
