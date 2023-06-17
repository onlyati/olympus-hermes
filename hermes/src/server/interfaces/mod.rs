pub mod classic;
pub mod grpc;
pub mod rest;
pub mod dummy;
pub mod websocket;

/// Interface handler
/// 
/// Task of interface handler is to start and monitor the specified interfaces like TCP, gRPC or REST.
/// Interface must implement ApplicationInterface trait to be able to compatible with this handler.
pub struct InterfaceHandler<T> {
    /// List about interfaces
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
    /// 
    /// # Parameters
    /// - `interface`: Interface that needs to be registered into this handler
    /// - `name`: Name of interface
    pub fn register_interface(&mut self, interface: T, name: String) {
        self.interfaces.push((name, interface));
    }

    /// Start each registered interface
    /// 
    /// # Details
    /// 
    /// When this function is called then `fn run()` function will be called with each of the interface.
    /// This function is implemented via `ApplicationInterface` trait.
    pub fn start(&mut self) {
        if self.interfaces.len() == 0 {
            tracing::error!("No interface is registered!");
            panic!("InterfaceHandler: No interface is registered!");
        }

        tracing::info!("Defined interfaces");

        for interface in &mut self.interfaces {
            tracing::info!("- {}", interface.0);
            interface.1.run();
        }
    }

    /// Monitor the interfaces
    /// 
    /// Monitor interfaces by an interval, if any interface failes then function return which lead for an application termination.
    pub async fn watch(&self) {
        let mut first_run = true;
        tokio::time::sleep(tokio::time::Duration::new(1, 0)).await;
        loop {
            for interface in &self.interfaces {
                match interface.1.is_it_run() {
                    Some(is_it_run) => {
                        if !is_it_run {
                            tracing::error!("'{}' has stopped", interface.0);
                            return;
                        }
                        if first_run && is_it_run {
                            tracing::info!("{}' is running!", interface.0);
                        }
                    }
                    None => {
                        tracing::error!("{}' has not been started", interface.0);
                        return;
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

/// Boxed implementation of ApplicationInterface trait. This is required because interfaces
/// are stored in heap (with Box allocation). They have to, because structs can have different size
/// so it is impossible to store them directly in a vector.
impl ApplicationInterface for Box<dyn ApplicationInterface> {
    fn run(&mut self) {
        return self.as_mut().run();
    }

    fn is_it_run(&self) -> Option<bool> {
        return self.as_ref().is_it_run();
    }
}
