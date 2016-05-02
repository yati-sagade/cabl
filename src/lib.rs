use std::io::Read;
use std::result;

type Result<T> = result::Result<T, String>;

struct Cabl<T: Iterator<Item=char>> {
    stream: T,
    look: Option<char>,
}

impl<T: Iterator<Item=char>> Cabl<T> {
    fn new(stream: T) -> Cabl<T> {
        let mut cabl = Cabl { stream: stream, look: None };
        cabl.read_next_char();
        cabl
    }

    fn read_next_char(&mut self) -> Option<char> {
        self.look = self.stream.next();
        self.look
    }

    fn error<X>(&self, msg: &str) -> Result<X> {
        Err(format!("Error: {}.", msg))
    }

    fn expected<X>(&self, what: &str) -> Result<X> {
        self.error(&format!("Expected {}", what))
    }

    fn get_name(&mut self) -> Result<char> {
        match self.look {
            Some(c) if c.is_alphabetic() => {
                self.read_next_char();
                Ok(c)
            }
            _ => self.expected("Name")
        }
    }

    fn get_number(&mut self) -> Result<char> {
        match self.look {
            Some(c) if c.is_digit(10) => {
                self.read_next_char();
                Ok(c)
            },
            _ => self.expected("Digit")
        }
    }

    fn emit(&self, msg: &str) {
        print!("\t{}", msg);
    }
        
    fn emitln(&self, msg: &str) {
        println!("\t{}", msg);
    }
}
