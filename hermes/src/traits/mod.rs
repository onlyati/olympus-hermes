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
