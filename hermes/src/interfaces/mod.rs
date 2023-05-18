pub struct InterfaceHandler<T> {
    interfaces: Vec<(String, T)>,
}

pub mod classic;
pub mod grpc;
pub mod rest;

impl<T: ApplicationInterface> InterfaceHandler<T> {
    pub fn new() -> Self {
        return Self {
            interfaces: Vec::new(),
        };
    }

    pub fn register_interface(&mut self, interface: T, name: String) {
        self.interfaces.push((name, interface));
    }

    pub fn start(&mut self) {
        if self.interfaces.len() == 0 {
            panic!("No interface is registered!");
        }

        for interface in &mut self.interfaces {
            interface.1.run();
        }
    }

    pub async fn watch(&self) {
        loop {
            for interface in &self.interfaces {
                match interface.1.is_it_run() {
                    Some(is_it_run) => {
                        if !is_it_run {
                            panic!("Interface '{}' has stopped", interface.0)
                        }
                    }
                    None => panic!("Interface '{}' has not been started", interface.0),
                }
            }
            tokio::time::sleep(tokio::time::Duration::new(5, 0)).await;
        }
    }
}

pub trait ApplicationInterface {
    fn run(&mut self);
    fn is_it_run(&self) -> Option<bool>;
}

impl ApplicationInterface for Box<dyn ApplicationInterface> {
    fn run(&mut self) {
        return self.as_mut().run();
    }

    fn is_it_run(&self) -> Option<bool> {
        return self.as_ref().is_it_run();
    }
}
