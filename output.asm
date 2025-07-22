bits 64
default rel
global main
extern printf

; --- Data section for string literals ---
section .data
    str_1: db "The integer is %d, the float is %.2f, and the char is %c.", 10, 0
    str_2: db "x is positive.", 10, 0
    str_0: db "Hello, world!", 10, 0

; --- Text section for executable code ---
section .text
main:
; --- Prologue et allocation de la pile ---
    push rbp
    mov rbp, rsp
; On alloue 48 octets : ~16 pour nos variables + 32 pour le "shadow space"
; IMPORTANT: Aligner la pile sur 16 octets avant les appels
    sub rsp, 48

; --- int x = 42; ---
    mov dword [rbp-4], 42
; --- float y = 3.14; ---
    mov rax, 4614253070214989087
    movq xmm0, rax
    movsd qword [rbp-12], xmm0
; --- char c = 'a'; ---
    mov byte [rbp-13], 'a'
; --- println("Hello, world!\n"); ---
; Aligner la pile avant l'appel (RSP doit être multiple de 16)
    and     rsp, ~15            ; Force l'alignement sur 16 octets
    sub rsp, 32

    mov rcx, str_0
    call printf

    add rsp, 32
; --- println("The integer is %d, the float is %f, and the char is %c.\n", x, y, c); ---
; Aligner la pile avant l'appel (RSP doit être multiple de 16)
    and     rsp, ~15            ; Force l'alignement sur 16 octets
    sub rsp, 32

    mov rcx, str_1
    mov     edx, [rbp-4]            ; Arg 2: la valeur de x (dans EDX)

; Pour le 3ème argument (flottant), il faut le mettre dans XMM2 ET dans R8D
    movsd   xmm2, [rbp-12]          ; Charge le flottant dans XMM2
    movq    r8, xmm2                ; ET copie la même valeur dans R8D

; Le 4ème argument va dans R9D
    movzx   r9d, byte [rbp-13]      ; Arg 4: la valeur de c (dans R9D)

    call printf

    add rsp, 32
; --- if (x > 0) ---
    mov     eax, [rbp-4]            ; Charge x dans eax pour la comparaison
    cmp eax, 0
    jle .else_block

; --- Bloc du "if" (si x > 0) ---
; --- println("x is positive.\n"); ---
; Aligner la pile avant l'appel (RSP doit être multiple de 16)
    and     rsp, ~15            ; Force l'alignement sur 16 octets
    sub rsp, 32

    mov rcx, str_2
    call printf

    add rsp, 32
; --- return x + 1; ---
    mov     eax, [rbp-4]            ; Recharge x dans eax
    inc eax
    jmp .end_program

.else_block:
; --- return 0; ---
; Ce bloc est exécuté si x <= 0
    xor eax, eax

.end_program:
; --- return 0; ---
    mov rax, 0
    ; result in eax
; --- Épilogue ---
    mov rsp, rbp
    pop rbp
    ret
