use std::result;
use std::process;
use std::collections::{HashSet,HashMap};

pub type Result<T> = result::Result<T, String>;


pub struct Cabl<T: Iterator<Item=char>> {
    stream:  T,
    look:    Option<char>,
    // List of variable declaration instructions. If nonempty, will be put
    // in the .data section.
    vardefs: HashSet<String>, 
    // Function name to the vector of lines that make up its body.
    fns:     HashMap<String, Vec<String>>,
}

impl<T: Iterator<Item=char>> Cabl<T> {
    pub fn new(stream: T) -> Cabl<T> {
        let mut cabl = Cabl { stream:  stream,
                              look:    None,
                              vardefs: HashSet::new(),
                              fns:     HashMap::new() };
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
        let have = char_or_msg(self.look, "EOF");
        self.error(&format!("Expected {} (found {})", what, have))
    }

    fn get_name(&mut self) -> Result<char> {
        let ret = if is_alphabetic(self.look) {
            self.look.ok_or(String::from("Inconsistent state"))
        } else {
            self.expected("Name")
        };
        self.read_next_char();
        ret
    }

    fn get_number(&mut self) -> Result<char> {
        let ret = if is_numeric(self.look) {
            self.look.ok_or(String::from("Inconsistent state"))
        } else {
            self.expected("Digit")
        };
        self.read_next_char();
        ret
    }

    fn emit(&self, msg: &str) {
        print!("\t{}", msg);
    }
        
    fn emitln(&self, msg: &str) {
        println!("\t{}", msg);
    }

    // <expression> ::= <addop> <term> 
    fn expression(&mut self) -> Result<()> {
        if is_addop(self.look) {
            // In case of a unary addop, just pretend that there is a ghost
            // operand = 0.
            self.emitln("xor eax, eax");
        } else {
            try!(self.term()); // parses and puts the number in eax
        }
        // Loop till we have an addop
        while is_addop(self.look) {
            self.emitln(&format!("push eax")); // put the numer in ebx
            match self.look {
                Some('+') => try!(self.add()),
                Some('-') => try!(self.sub()),
                _         => return self.expected::<()>("Addop")
            };
        }
        Ok(())
    }

    // <term> ::= <factor> [<mulop> <factor>]*
    // A term is either a single factor, or a product of n factors (with
    // "product" being a combination of two factors using "*" or "/").
    fn term(&mut self) -> Result<()> {
        try!(self.factor());
        while is_mulop(self.look) {
            self.emitln(&format!("push eax"));
            match self.look {
                Some('*') => try!(self.mul()),
                Some('/') => try!(self.div()),
                _         => return self.expected::<()>("Mulop")
            };
        }
        Ok(())
    }

    // <factor> ::= <number> | '(' <expression> ')' | <ident>
    fn factor(&mut self) -> Result<()> {
        if is_lparen(self.look) {
            try!(self.Match('('));
            try!(self.expression());
            try!(self.Match(')'));
        } else if is_numeric(self.look) {
            let n = try!(self.get_number());
            self.emitln(&format!("mov eax, {}", n));
        } else if is_alphabetic(self.look) {
            try!(self.ident());
        }
        Ok(())
    }

    // <ident> ::= <variable> | <funcall>
    fn ident(&mut self) -> Result<()> {
        let v = try!(self.get_name());
        if is_lparen(self.look) {
            try!(self.Match('('));
            try!(self.Match(')'));
            self.emitln(&format!("call {}", v));
            // Insert dummy code for the function. TODO: Take this bit out.
            self.fns.insert(format!("{}", v), vec![String::from("ret")]);
        } else {
            self.emitln(&format!("mov eax, [{}]", v));
            self.vardefs.insert(format!("{} dd 0x00", v));
        }
        Ok(())
    }

    // <assignment> ::= <ident> '=' <expression>
    fn assignment(&mut self) -> Result<()> {
        let v = try!(self.get_name());
        self.vardefs.insert(format!("{} dd 0x00", v));
        try!(self.Match('='));
        try!(self.expression());
        self.emitln(&format!("mov [{}], eax", v));
        Ok(())
    }

    fn add(&mut self) -> Result<()> {
        try!(self.Match('+'));
        try!(self.term()); // the second operand of addition will in eax (the first was put in ebx by expression())

        self.emitln("pop ebx");
        self.emitln("add eax, ebx"); // the result will be in eax

        Ok(())
    }

    fn sub(&mut self) -> Result<()> {
        try!(self.Match('-'));
        try!(self.term()); // the second operand of addition will in eax (the first was put in ebx by expression())

        self.emitln("pop ebx");
        self.emitln("sub eax, ebx"); // the result will be in eax
        self.emitln("neg eax ;; We have subtracted the first operand from the second, so negate eax");

        Ok(())
    }

    fn div(&mut self) -> Result<()> {
        try!(self.Match('/'));
        try!(self.factor());

        self.emitln("mov ebx, eax ;; div: Move the second arg to ebx");
        self.emitln("pop eax      ;; div: Now eax has the first arg");
        self.emitln("div ebx ;; eax <- eax / ebx");

        Ok(())
    }

    fn mul(&mut self) -> Result<()> {
        try!(self.Match('*'));
        try!(self.factor());

        self.emitln("pop ebx");
        self.emitln("mul ebx ;; eax <- eax * ebx");

        Ok(())
    }

    fn Match(&mut self, c: char) -> Result<()> {
        match self.look {
            Some(v) if v == c => {
                self.read_next_char();
                Ok(())
            },
            _       => self.expected(&format!("{}", c))
        }
    }


    pub fn process(&mut self) {
        self.prelude(); // The section declaration and beginning of the entry point.
        let result = self.assignment(); // Actual code gen
        self.abort_on_error(&result);
        if !is_newline(self.look) {
            println!("Bad expression. Expected newline, found {}",
                     char_or_msg(self.look, "EOF"));
            process::exit(1);
        }
        self.closing();
    }

    fn prelude(&self) {
        println!(concat!("section .text\n",
                         "global _start ;; _start for GCC\n",
                         "bits 32 ;; push is not supported in 64 bit mode\n",
                         "_start:"));
    }

    fn closing(&self) {
        self.emitln("");
        self.emitln("mov eax, 0x1 ;; Exit syscall code.");
        self.emitln("int 0x80     ;; Interrupt to syscall.");
        self.emitln("");

        // Emit function definitions
        for (name, lines) in &self.fns {
            println!("{}:", name);
            for line in lines {
                self.emitln(line); 
            }
            println!("");
        }

        // Emit the .data section if needed.
        if self.vardefs.len() > 0 {
            println!("section .data");
            for line in &self.vardefs {
                self.emitln(line);
            }
        }
        
    }

    fn abort_on_error<X>(&self, result: &Result<X>) {
        match result {
            &Err(ref s) => {
                println!("\n{}\nAborting due to previous errors.", s);
                process::exit(1);
            },
            &Ok(_)      => { },
        } 
    }
}

fn is_addop(c: Option<char>) -> bool { c.map_or(false, |c| c == '+' || c == '-') }

fn is_mulop(c: Option<char>) -> bool { c.map_or(false, |c| c == '*' || c == '/') }

fn is_numeric(c: Option<char>) -> bool { c.map_or(false, |c| c.is_digit(10)) }

fn is_alphabetic(c: Option<char>) -> bool { c.map_or(false, |c| c.is_alphabetic()) }

fn is_lparen(c: Option<char>) -> bool { c.map_or(false, |c| c == '(')  }

fn is_newline(c: Option<char>) -> bool { c.map_or(false, |c| c == '\n') }

fn char_or_msg(c: Option<char>, msg: &str) -> String {
    c.map_or(String::from(msg), |c| format!("'{}'", c))
}
