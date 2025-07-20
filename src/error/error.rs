use std::fmt;

/// Types d'erreurs du compilateur
#[derive(Debug, Clone)]
pub enum CompilerError {
    /// Erreurs lexicales
    LexError {
        message: String,
        line: usize,
        column: usize,
    },
    /// Erreurs syntaxiques
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },
    /// Erreurs sémantiques
    SemanticError {
        message: String,
        line: usize,
        column: usize,
    },
    /// Erreurs de génération de code
    CodegenError {
        message: String,
    },
    /// Erreurs d'entrée/sortie
    IoError {
        message: String,
    },
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CompilerError::LexError { message, line, column } => {
                write!(f, "Erreur lexicale à {}:{}: {}", line, column, message)
            }
            CompilerError::ParseError { message, line, column } => {
                write!(f, "Erreur de syntaxe à {}:{}: {}", line, column, message)
            }
            CompilerError::SemanticError { message, line, column } => {
                write!(f, "Erreur sémantique à {}:{}: {}", line, column, message)
            }
            CompilerError::CodegenError { message } => {
                write!(f, "Erreur de génération de code: {}", message)
            }
            CompilerError::IoError { message } => {
                write!(f, "Erreur d'E/S: {}", message)
            }
        }
    }
}

impl std::error::Error for CompilerError {}

impl From<std::io::Error> for CompilerError {
    fn from(err: std::io::Error) -> Self {
        CompilerError::IoError {
            message: err.to_string(),
        }
    }
}