
use std::fmt;

#[derive(Debug, Clone)]
pub struct CompilerError {
    pub kind: ErrorKind,
    pub span: Span,
    pub source_context: SourceContext,
    pub suggestions: Vec<Suggestion>,
    pub severity: Severity,
}

#[derive(Debug, Clone)]
pub enum ErrorKind {
    Lexical(LexicalError),
    Syntactic(SyntacticError),
    Semantic(SemanticError),
    Codegen(CodegenError),
    Internal(InternalError),
}

#[derive(Debug, Clone)]
pub enum Severity {
    Error,
    Warning,
    Note,
    Help,
}

#[derive(Debug, Clone)]
pub struct Span {
    pub start: Position,
    pub end: Position,
    pub source_id: SourceId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceId(pub String);

#[derive(Debug, Clone)]
pub struct SourceContext {
    pub source_id: SourceId,
    pub source_text: String,
    pub line_starts: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub span: Option<Span>,
    pub replacement: Option<String>,
    pub suggestion_type: SuggestionType,
}

#[derive(Debug, Clone)]
pub enum SuggestionType {
    Replace,
    Insert,
    Remove,
    Note,
}

#[derive(Debug, Clone)]
pub enum LexicalError {
    UnexpectedCharacter(char),
    UnterminatedString,
    UnterminatedComment,
    InvalidNumber(String),
    InvalidEscape(char),
}

#[derive(Debug, Clone)]
pub enum SyntacticError {
    UnexpectedToken {
        expected: Vec<String>,
        found: String,
    },
    MissingToken(String),
    ExtraToken(String),
    InvalidExpression,
    InvalidStatement,
    UnmatchedDelimiter {
        opening: char,
        expected_closing: char,
        found: Option<char>,
    },
}

#[derive(Debug, Clone)]
pub enum SemanticError {
    UndefinedVariable(String),
    UndefinedFunction(String),
    TypeMismatch {
        expected: String,
        found: String,
    },
    RedefinedSymbol {
        name: String,
        original_span: Span,
    },
    InvalidOperation {
        operation: String,
        operand_types: Vec<String>,
    },
    InvalidAssignment {
        target_type: String,
        value_type: String,
    },
    UnreachableCode,
    MissingReturn,
}

#[derive(Debug, Clone)]
pub enum CodegenError {
    UnsupportedFeature(String),
    RegisterAllocationFailed,
    InvalidInstruction(String),
    TargetSpecificError(String),
}

#[derive(Debug, Clone)]
pub enum InternalError {
    CompilerBug(String),
    OutOfMemory,
    IoError(String),
}

pub struct ErrorReporter {
    source_manager: SourceManager,
    error_count: usize,
    warning_count: usize,
}

pub struct SourceManager {
    sources: std::collections::HashMap<SourceId, SourceContext>,
}

impl ErrorReporter {
    pub fn new() -> Self {
        Self {
            source_manager: SourceManager::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    pub fn add_source(&mut self, source_id: SourceId, content: String) {
        let line_starts = Self::compute_line_starts(&content);
        let context = SourceContext {
            source_id: source_id.clone(),
            source_text: content,
            line_starts,
        };
        self.source_manager.sources.insert(source_id, context);
    }

    pub fn report_error(&mut self, error: CompilerError) {
        match error.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
            _ => {}
        }

        self.print_error(&error);
    }

    fn print_error(&self, error: &CompilerError) {
        println!("{}: {}", self.severity_prefix(&error.severity), error);

        if let Some(context) = self.source_manager.sources.get(&error.span.source_id) {
            self.print_source_context(context, &error.span);
        }

        for suggestion in &error.suggestions {
            println!("  {}: {}", self.suggestion_prefix(&suggestion.suggestion_type), suggestion.message);
        }
    }

    fn print_source_context(&self, context: &SourceContext, span: &Span) {
        let start_line = span.start.line;
        let end_line = span.end.line;

        let context_lines = 2;
        let first_line = start_line.saturating_sub(context_lines);
        let last_line = (end_line + context_lines).min(context.line_starts.len().saturating_sub(1));

        for line_num in first_line..=last_line {
            let line_content = self.get_line_content(context, line_num);
            let line_number_width = (last_line + 1).to_string().len();

            if line_num >= start_line && line_num <= end_line {
                println!("{:width$} | {}", line_num + 1, line_content, width = line_number_width);
                
                if line_num == start_line {
                    let start_col = if line_num == start_line { span.start.column } else { 0 };
                    let end_col = if line_num == end_line { span.end.column } else { line_content.len() };
                    
                    print!("{:width$} | ", "", width = line_number_width);
                    for i in 0..line_content.len() {
                        if i >= start_col && i < end_col {
                            print!("^");
                        } else {
                            print!(" ");
                        }
                    }
                    println!();
                }
            } else {
                println!("{:width$} | {}", line_num + 1, line_content, width = line_number_width);
            }
        }
    }

    fn get_line_content(&self, context: &SourceContext, line_num: usize) -> &str {
        if line_num >= context.line_starts.len() {
            return "";
        }

        let start = context.line_starts[line_num];
        let end = if line_num + 1 < context.line_starts.len() {
            context.line_starts[line_num + 1].saturating_sub(1) // Exclude newline
        } else {
            context.source_text.len()
        };

        &context.source_text[start..end]
    }

    fn compute_line_starts(content: &str) -> Vec<usize> {
        let mut line_starts = vec![0];
        for (i, ch) in content.char_indices() {
            if ch == '\n' {
                line_starts.push(i + 1);
            }
        }
        line_starts
    }

    fn severity_prefix(&self, severity: &Severity) -> &'static str {
        match severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Note => "note",
            Severity::Help => "help",
        }
    }

    fn suggestion_prefix(&self, suggestion_type: &SuggestionType) -> &'static str {
        match suggestion_type {
            SuggestionType::Replace => "suggestion",
            SuggestionType::Insert => "help",
            SuggestionType::Remove => "help",
            SuggestionType::Note => "note",
        }
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }
}

impl SourceManager {
    pub fn new() -> Self {
        Self {
            sources: std::collections::HashMap::new(),
        }
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Lexical(err) => write!(f, "lexical error: {}", err),
            ErrorKind::Syntactic(err) => write!(f, "syntax error: {}", err),
            ErrorKind::Semantic(err) => write!(f, "semantic error: {}", err),
            ErrorKind::Codegen(err) => write!(f, "code generation error: {}", err),
            ErrorKind::Internal(err) => write!(f, "internal compiler error: {}", err),
        }
    }
}

impl fmt::Display for LexicalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LexicalError::UnexpectedCharacter(ch) => write!(f, "unexpected character '{}'", ch),
            LexicalError::UnterminatedString => write!(f, "unterminated string literal"),
            LexicalError::UnterminatedComment => write!(f, "unterminated comment"),
            LexicalError::InvalidNumber(num) => write!(f, "invalid number '{}'", num),
            LexicalError::InvalidEscape(ch) => write!(f, "invalid escape sequence '\\{}'", ch),
        }
    }
}

impl fmt::Display for SyntacticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyntacticError::UnexpectedToken { expected, found } => {
                write!(f, "expected {}, found '{}'", expected.join(" or "), found)
            }
            SyntacticError::MissingToken(token) => write!(f, "missing '{}'", token),
            SyntacticError::ExtraToken(token) => write!(f, "unexpected '{}'", token),
            SyntacticError::InvalidExpression => write!(f, "invalid expression"),
            SyntacticError::InvalidStatement => write!(f, "invalid statement"),
            SyntacticError::UnmatchedDelimiter { opening, expected_closing, found } => {
                match found {
                    Some(found_char) => write!(f, "mismatched delimiter: expected '{}' to close '{}', found '{}'", expected_closing, opening, found_char),
                    None => write!(f, "unclosed delimiter: expected '{}' to close '{}'", expected_closing, opening),
                }
            }
        }
    }
}

impl fmt::Display for SemanticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SemanticError::UndefinedVariable(name) => write!(f, "undefined variable '{}'", name),
            SemanticError::UndefinedFunction(name) => write!(f, "undefined function '{}'", name),
            SemanticError::TypeMismatch { expected, found } => {
                write!(f, "type mismatch: expected '{}', found '{}'", expected, found)
            }
            SemanticError::RedefinedSymbol { name, .. } => write!(f, "redefinition of '{}'", name),
            SemanticError::InvalidOperation { operation, operand_types } => {
                write!(f, "invalid operation '{}' for types [{}]", operation, operand_types.join(", "))
            }
            SemanticError::InvalidAssignment { target_type, value_type } => {
                write!(f, "cannot assign '{}' to '{}'", value_type, target_type)
            }
            SemanticError::UnreachableCode => write!(f, "unreachable code"),
            SemanticError::MissingReturn => write!(f, "missing return statement"),
        }
    }
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::UnsupportedFeature(feature) => write!(f, "unsupported feature: {}", feature),
            CodegenError::RegisterAllocationFailed => write!(f, "register allocation failed"),
            CodegenError::InvalidInstruction(instr) => write!(f, "invalid instruction: {}", instr),
            CodegenError::TargetSpecificError(msg) => write!(f, "target-specific error: {}", msg),
        }
    }
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InternalError::CompilerBug(msg) => write!(f, "compiler bug: {}", msg),
            InternalError::OutOfMemory => write!(f, "out of memory"),
            InternalError::IoError(msg) => write!(f, "I/O error: {}", msg),
        }
    }
}

impl std::error::Error for CompilerError {}

impl CompilerError {
    pub fn lexical_error(error: LexicalError, span: Span, context: SourceContext) -> Self {
        Self {
            kind: ErrorKind::Lexical(error),
            span,
            source_context: context,
            suggestions: Vec::new(),
            severity: Severity::Error,
        }
    }

    pub fn syntax_error(error: SyntacticError, span: Span, context: SourceContext) -> Self {
        Self {
            kind: ErrorKind::Syntactic(error),
            span,
            source_context: context,
            suggestions: Vec::new(),
            severity: Severity::Error,
        }
    }

    pub fn semantic_error(error: SemanticError, span: Span, context: SourceContext) -> Self {
        Self {
            kind: ErrorKind::Semantic(error),
            span,
            source_context: context,
            suggestions: Vec::new(),
            severity: Severity::Error,
        }
    }

    pub fn with_suggestion(mut self, suggestion: Suggestion) -> Self {
        self.suggestions.push(suggestion);
        self
    }

    pub fn with_severity(mut self, severity: Severity) -> Self {
        self.severity = severity;
        self
    }
}
