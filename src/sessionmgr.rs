use logind_zbus::manager::{IsSupported, ManagerProxyBlocking};
use zbus::blocking::Connection;

#[derive(Debug, Clone)]
pub enum SessionOperation {
    Suspend,
    Reboot,
    PoweOff,
}

#[derive(Debug)]
pub struct SessionMgr {
    pub enable_suspend: bool,
    pub enable_reboot: bool,
    pub enable_poweroff: bool,

    connection: Connection,
}

impl SessionMgr {
    // TODO: handle errors here more gracefully,
    //       if the connection fails should only not display related options
    pub fn new() -> Self {
        let connection = Connection::system().expect("unable to connectt to system DBus");
        let mgr = ManagerProxyBlocking::new(&connection)
            .expect("unable to connect to logind DBus interface");

        Self {
            enable_suspend: mgr.can_suspend().unwrap() == IsSupported::Yes,
            enable_reboot: mgr.can_reboot().unwrap() == IsSupported::Yes,
            enable_poweroff: mgr.can_power_off().unwrap() == IsSupported::Yes,

            connection,
        }
    }

    pub fn perform(&self, op: &SessionOperation) {
        let mgr = ManagerProxyBlocking::new(&self.connection)
            .expect("unable to connect to logind DBus interface");

        // TODO: should I treat this errors in a better way?
        match op {
            SessionOperation::Suspend => {
                assert!(self.enable_suspend);
                mgr.suspend(false).expect("unable to request suspend")
            }
            SessionOperation::Reboot => {
                assert!(self.enable_reboot);
                mgr.reboot(false).expect("unable to request reboot")
            }
            SessionOperation::PoweOff => {
                assert!(self.enable_poweroff);
                mgr.power_off(false).expect("unable to request power off")
            }
        }
    }
}
