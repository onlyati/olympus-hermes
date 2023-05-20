pub mod classic;
pub mod grpc;
pub mod rest;
pub mod dummy;

/// # Interface handler
/// 
/// Task of interface handler is to start and monitor the specified interfaces like TCP, gRPC or REST.
/// Interface must implement ApplicationInterface trait.
pub struct InterfaceHandler<T> {
    interfaces: Vec<(String, T)>,
}

impl<T: ApplicationInterface> InterfaceHandler<T> {
    /// Allocate new interface handler
    pub fn new() -> Self {
        return Self {
            interfaces: Vec::new(),
        };
    }

    /// Function to register interfaces that applied ApplicationInterface trait
    pub fn register_interface(&mut self, interface: T, name: String) {
        self.interfaces.push((name, interface));
    }

    /// Start each registered interface
    pub fn start(&mut self) {
        if self.interfaces.len() == 0 {
            log::error!("InterfaceHandler: No interface is registered!");
            panic!("InterfaceHandler: No interface is registered!");
        }

        log::info!("InterfaceHandler: Defined interfaces");

        for interface in &mut self.interfaces {
            log::info!("InterfaceHandler: - {}", interface.0);
            interface.1.run();
        }
    }

    /// Monitor them by an interval, if any interface failes then program has  apanic reaction
    pub async fn watch(&self) {
        let mut first_run = true;
        tokio::time::sleep(tokio::time::Duration::new(1, 0)).await;
        loop {
            for interface in &self.interfaces {
                match interface.1.is_it_run() {
                    Some(is_it_run) => {
                        if !is_it_run {
                            log::error!("InterfaceHandler: '{}' has stopped", interface.0);
                            panic!("InterfaceHandler: '{}' has stopped", interface.0);
                        }
                        if first_run && is_it_run {
                            log::info!("InterfaceHandler: '{}' is running!", interface.0);
                        }
                    }
                    None => {
                        log::error!("InterfaceHandler: '{}' has not been started", interface.0);
                        panic!("InterfaceHandler: '{}' has not been started", interface.0);
                    },
                }
            }
            first_run = false;
            tokio::time::sleep(tokio::time::Duration::new(5, 0)).await;
        }
    }
}

/// Trait that must be implemented that an interface will be able to use InterfaceHandler
pub trait ApplicationInterface {
    fn run(&mut self);
    fn is_it_run(&self) -> Option<bool>;
}

/// Boxed implementation of ApplicationInterface trait
impl ApplicationInterface for Box<dyn ApplicationInterface> {
    fn run(&mut self) {
        return self.as_mut().run();
    }

    fn is_it_run(&self) -> Option<bool> {
        return self.as_ref().is_it_run();
    }
}
