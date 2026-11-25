; Computes 10! with a recursive function and prints it.
        push 10
        call fact
        print
        halt

; fact(n): n on the stack -> n! on the stack
fact:   lstore 0
        lload 0
        push 2
        lt
        jz recurse      ; n >= 2
        push 1
        ret
recurse: lload 0
        lload 0
        push 1
        sub
        call fact
        mul
        ret
