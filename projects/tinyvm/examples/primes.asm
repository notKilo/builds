; Prints every prime below LIMIT using trial division.
.const LIMIT 50

        push 2
        lstore 0            ; n
next:   lload 0
        push LIMIT
        lt
        jz end
        push 2
        lstore 1            ; d
trial:  lload 1
        lload 1
        mul
        lload 0
        gt
        jnz prime           ; d*d > n  => prime
        lload 0
        lload 1
        mod
        jz composite
        lload 1
        push 1
        add
        lstore 1
        jmp trial
prime:  lload 0
        print
composite:
        lload 0
        push 1
        add
        lstore 0
        jmp next
end:    halt
