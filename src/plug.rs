use std::io::Read;

use libloading::{Library, Symbol};

pub struct Plug<T> {
    pub data: *mut T,
    pub dll: Library,
    pub path: String,
    pub symbol: String,
}

impl<T> Plug<T> {
    pub fn new(path: &str, symbol: &str) -> Self {
        unsafe {
            let dll = Library::new(path).expect("Dll Not Found");
            let data: Symbol<extern "C" fn() -> *mut T> =
                dll.get(symbol.as_bytes()).expect("Symbol does not exist!");
            Self {
                data: data(),
                dll,
                path: path.to_string(),
                symbol: symbol.to_string(),
            }
        }
    }

    pub fn reload(&mut self) {
        unsafe {
            drop(Box::from_raw(self.data));
            let dll =
                Library::new(&self.path).expect(&format!("Dll not found! for: {}", self.path));
            let data: Symbol<extern "C" fn() -> *mut T> = dll
                .get(self.symbol.as_bytes())
                .expect(&format!("Symbol does not exist! for: {}", self.symbol));
            self.data = data();
            self.dll = dll;
        }
    }
}
