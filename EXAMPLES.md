# Wheel Language - Demonstração Prática

## O que é Wheel?

Wheel é uma linguagem de programação compilada, de nível médio, escrita em Rust que gera executáveis x86_64 independentes (sem dependência de Rust, Python ou outras linguagens de runtime).

## Status Atual

✅ **Funcionalidades Implementadas:**
- Lexer completo (strings, inteiros, operadores, keywords)
- Parser recursivo descendente (let, func, print, expressões)
- Codegen em assembly x86_64 (syscall write/exit)
- CLI com geração de executáveis (-o, --mode ge/gb)
- Exemplos funcionais (hello, math, calculadora)
- Constantes e variáveis com contexto de compile-time

⏳ **Em Progresso:**
- Sistema de imports (framework em desenvolvimento)
- ELF writer nativo (machine code sem intermediário gcc)

❌ **Não Implementado:**
- input() para leitura de stdin
- Condicionais (if/else)
- Loops (while/for)
- Arrays/Structs
- Suporte a múltiplas arquiteturas

## Exemplos de Uso

### Hello World
```wheel
print("Hello, Wheel world!");
```

Compilar:
```bash
./wheelc examples/hello.wheel -o hello --mode ge
./hello
# Output: Hello, Wheel world!
```

### Aritmética
```wheel
let a = 2;
let b = 3;
print(a + b);  # 5
print(a * b);  # 6
```

### Calculadora
```wheel
let num1 = 10;
let num2 = 5;
print("10 + 5 = ");
print(num1 + num2);
print("\n");
print("10 * 5 = ");
print(num1 * num2);
```

## Arquitetura

```
Source (.wheel)
    ↓ (Lexer: tokenização)
Tokens
    ↓ (Parser: análise sintática)
AST (Abstract Syntax Tree)
    ↓ (Codegen: geração de assembly x86_64)
Assembly (.s)
    ↓ (gcc/clang: montagem e linkagem)
ELF Executable
    ↓
./executable
```

## Compile & Run

```bash
# Build the compiler
cargo build --release

# Compile a Wheel program
./target/release/wheelc program.wheel -o program --mode ge

# Run the generated executable
./program
```

## Próximas Melhorias (Roadmap)

1. **Imports & Modularity** - Permitir dividir código em múltiplos arquivos
2. **Input() & I/O** - Leitura de stdin e arquivo
3. **Condicionais** - if/else/match statements
4. **Loops** - for/while statements  
5. **Tipos Complexos** - Arrays, structs, enums
6. **Machine Code Generation** - Eliminar dependência de gcc (ELF writer nativo)
7. **Multi-Architecture** - Suporte a ARM64, x86 de 32 bits, Windows PE
8. **Standard Library Expandida** - math.sqrt(), math.sin(), os.read_file(), etc.

## Exemplos no Repositório

- `examples/hello.wheel` - Hello World básico
- `examples/math_example.wheel` - Aritmética com variáveis
- `examples/calculator.wheel` - Calculadora simples  
- `examples/utils.wheel` - Exemplo de lib (import framework)
- `examples/interactive.wheel` - Exemplo com input() (não implementado ainda)

## Linguagem Syntax

```ebnf
Program     ::= Stmt*
Stmt        ::= Let | Func | Return | Import | Expr ";"
Let         ::= "let" Ident "=" Expr ";"
Func        ::= "func" Ident "(" Ident* ")" "{" Stmt* "}"
Return      ::= "return" Expr? ";"
Import      ::= "import" String ";"
Expr        ::= BinaryOp | Call | Atom
BinaryOp   ::= Expr ("+" | "-" | "*" | "/") Expr
Call        ::= Ident "(" Expr* ")"
Atom        ::= Int | String | Ident | "(" Expr ")"
```

## Dificuldades Resolvidas

1. ✅ String literals com RIP-relative addressing
2. ✅ Constant folding com contexto de variáveis
3. ✅ Deduplicação de strings no rodata
4. ✅ Syscall write/exit x86_64

## Ferramentas Utilizadas

- **Rust** - Implementação do compilador
- **x86_64 Assembly** - Geração de código alvo
- **gcc/clang** - Linkagem (temporária)
- **Linux syscalls** - write(), exit()

---

Para mais informações, consulte o [ROADMAP.md](./ROADMAP.md)
