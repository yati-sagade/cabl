cabl
=====

"cabl" is the reverse of "lbac", which is an excellent tutorial on compiler
construction by Jack Crenshaw: http://compilers.iecc.com/crenshaw/

Running
=======

- Get Rust nightly.
- Clone this repo:
    
    git clone git@github.com:yati-sagade/cabl.git

- Change directories and run:
    
    cd cabl
    cargo run
         Running `target/debug/cabl`
    section .text
    global _start ;; _start for GCC
    bits 32 ;; push is not supported in 64 bit mode
    _start:
        mov eax, 1
        push eax
        mov eax, 1
        push eax
        call e
        pop ebx
        add eax, ebx
        mov ebx, eax ;; div: Move the second arg to ebx
        pop eax      ;; div: Now eax has the first arg
        div ebx ;; eax <- eax / ebx
        mov [x], eax
        
        mov eax, 0x1 ;; Exit syscall code.
        int 0x80     ;; Interrupt to syscall.
        
    e:
        ret
    
    section .data
        x dd 0x00

