use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }
    
    pub fn dummy() -> Self {
        Self { start: 0, end: 0, line: 1, column: 1 }
    }
}

#[derive(Debug, Clone)]
pub struct SourceContext {
    pub filename: String,
    pub source: String,
    pub span: Span,
}

impl SourceContext {
    pub fn new(filename: String, source: String, span: Span) -> Self {
        Self { filename, source, span }
    }
    
    pub fn get_line(&self) -> Option<&str> {
        self.source.lines().nth(self.span.line.saturating_sub(1))
    }
    
    pub fn get_context_lines(&self, context: usize) -> Vec<(usize, &str)> {
        let start_line = self.span.line.saturating_sub(context + 1);
        let end_line = self.span.line + context;
        
        self.source
            .lines()
            .enumerate()
            .skip(start_line)
            .take(end_line - start_line)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub span: Option<Span>,
    pub replacement: Option<String>,
}

impl Suggestion {
    pub fn new(message: String) -> Self {
        Self { message, span: None, replacement: None }
    }
    
    pub fn with_replacement(message: String, span: Span, replacement: String) -> Self {
        Self { message, span: Some(span), replacement: Some(replacement) }
    }
}

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub span: Span,
    pub source_context: Option<SourceContext>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Lexical(LexicalError),
    Syntactic(SyntacticError),
    Semantic(SemanticError),
    Codegen(CodegenError),
    Io(String),
}

#[derive(Debug, Clone)]
pub enum LexicalError {
    UnexpectedCharacter(char),
    UnterminatedString,
    InvalidNumber(String),
    InvalidEscape(char),
    Generic(String),
}

#[derive(Debug, Clone)]
pub enum SyntacticError {
    UnexpectedToken(String),
    MissingToken(String),
    InvalidExpression,
    UnmatchedDelimiter(char),
    Generic(String),
}

#[derive(Debug, Clone)]
pub enum SemanticError {
    UndefinedVariable(String),
    TypeMismatch { expected: String, found: String },
    RedefinedVariable(String),
    InvalidOperation(String),
    Generic(String),
}

#[derive(Debug, Clone)]
pub enum CodegenError {
    UnsupportedFeature(String),
    RegisterAllocation(String),
    InvalidInstruction(String),
    Generic(String),
}

impl CompilerError {
    pub fn lexical(error: LexicalError, span: Span) -> Self {
        Self {
            kind: ErrorKind::Lexical(error),
            span,
            source_context: None,
            suggestions: Vec::new(),
        }
    }
    
    pub fn syntactic(error: SyntacticError, span: Span) -> Self {
        Self {
            kind: ErrorKind::Syntactic(error),
            span,
            source_context: None,
            suggestions: Vec::new(),
        }
    }
    
    pub fn semantic(error: SemanticError, span: Span) -> Self {
        Self {
            kind: ErrorKind::Semantic(error),
            span,
            source_context: None,
            suggestions: Vec::new(),
        }
    }
    
    pub fn codegen(error: CodegenError, span: Span) -> Self {
        Self {
            kind: ErrorKind::Codegen(error),
            span,
            source_context: None,
            suggestions: Vec::new(),
        }
    }
    
    pub fn io(message: String) -> Self {
        Self {
            kind: ErrorKind::Io(message),
            span: Span::dummy(),
            source_context: None,
            suggestions: Vec::new(),
        }
    }
    
    pub fn with_context(mut self, context: SourceContext) -> Self {
        self.source_context = Some(context);
        self
    }
    
    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }
    
    pub fn with_suggestions(mut self, suggestions: Vec<Suggestion>) -> Self {
        self.suggestions.extend(suggestions);
        self
    }
    
    pub fn lex_error(message: String, line: usize, column: usize) -> Self {
        Self::lexical(
            LexicalError::Generic(message),
            Span::new(0, 0, line, column)
        )
    }
    
    pub fn parse_error(message: String, line: usize, column: usize) -> Self {
        Self::syntactic(
            SyntacticError::Generic(message),
            Span::new(0, 0, line, column)
        )
    }
    
    pub fn semantic_error(message: String, line: usize, column: usize) -> Self {
        Self::semantic(
            SemanticError::Generic(message),
            Span::new(0, 0, line, column)
        )
    }
    
    pub fn codegen_error(message: String) -> Self {
        Self::codegen(
            CodegenError::Generic(message),
            Span::dummy()
        )
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Lexical(err) => write!(f, "Lexical error: {}", err),
            ErrorKind::Syntactic(err) => write!(f, "Syntax error: {}", err),
            ErrorKind::Semantic(err) => write!(f, "Semantic error: {}", err),
            ErrorKind::Codegen(err) => write!(f, "Code generation error: {}", err),
            ErrorKind::Io(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexicalError::UnexpectedCharacter(ch) => write!(f, "unexpected character '{}'", ch),
            LexicalError::UnterminatedString => write!(f, "unterminated string literal"),
            LexicalError::InvalidNumber(num) => write!(f, "invalid number '{}'", num),
            LexicalError::InvalidEscape(ch) => write!(f, "invalid escape sequence '\\{}'", ch),
            LexicalError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl fmt::Display for SyntacticError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SyntacticError::UnexpectedToken(token) => write!(f, "unexpected token '{}'", token),
            SyntacticError::MissingToken(token) => write!(f, "expected '{}'", token),
            SyntacticError::InvalidExpression => write!(f, "invalid expression"),
            SyntacticError::UnmatchedDelimiter(delim) => write!(f, "unmatched delimiter '{}'", delim),
            SyntacticError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            SemanticError::UndefinedVariable(name) => write!(f, "undefined variable '{}'", name),
            SemanticError::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected '{}', found '{}'", expected, found)
            }
            SemanticError::RedefinedVariable(name) => write!(f, "variable '{}' is already defined", name),
            SemanticError::InvalidOperation(op) => write!(f, "invalid operation '{}'", op),
            SemanticError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CodegenError::UnsupportedFeature(feature) => write!(f, "unsupported feature '{}'", feature),
            CodegenError::RegisterAllocation(msg) => write!(f, "register allocation error: {}", msg),
            CodegenError::InvalidInstruction(instr) => write!(f, "invalid instruction '{}'", instr),
            CodegenError::Generic(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CompilerError {}

impl From<std::io::Error> for CompilerError {
    fn from(err: std::io::Error) -> Self {
        CompilerError::io(err.to_string())
    }
}

pub struct ErrorReporter {
    pub show_colors: bool,
    pub show_context: bool,
    pub context_lines: usize,
}

impl Default for ErrorReporter {
    fn default() -> Self {
        Self {
            show_colors: true,
            show_context: true,
            context_lines: 2,
        }
    }
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self::default()
    }
    
    pub fn report(&self, error: &CompilerError) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("error: {}\n", error));
        
        if let Some(context) = &error.source_context {
            output.push_str(&format!("  --> {}:{}:{}\n", 
                context.filename, error.span.line, error.span.column));
            
            if self.show_context {
                output.push_str(&self.format_source_context(context, &error.span));
            }
        } else {
            output.push_str(&format!("  at line {}, column {}\n", 
                error.span.line, error.span.column));
        }
        
        if !error.suggestions.is_empty() {
            output.push_str("\nhelp:\n");
            for suggestion in &error.suggestions {
                output.push_str(&format!("  {}\n", suggestion.message));
            }
        }
        
        output
    }
    
    fn format_source_context(&self, context: &SourceContext, span: &Span) -> String {
        let mut output = String::new();
        let context_lines = context.get_context_lines(self.context_lines);
        
        for (line_num, line_content) in context_lines {
            let line_number = line_num + 1;
            output.push_str(&format!("{:4} | {}\n", line_number, line_content));
            
            if line_number == span.line {
                output.push_str("     | ");
                for _ in 0..span.column.saturating_sub(1) {
                    output.push(' ');
                }
                for _ in span.start..span.end.min(span.start + line_content.len()) {
                    output.push('^');
                }
                output.push('\n');
            }
        }
        
        output
    }
}
