; Prints the first 10 Fibonacci numbers.
.const COUNT 10

        push 0
        lstore 0        ; a
        push 1
        lstore 1        ; b
        push COUNT
        lstore 2        ; remaining
loop:   lload 2
        jz done
        lload 0
        print
        lload 0
        lload 1
        add             ; a + b
        lload 1
        lstore 0        ; a = b
        lstore 1        ; b = a + b
        lload 2
        push 1
        sub
        lstore 2
        jmp loop
done:   halt
