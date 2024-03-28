use crate::{
    assembler::{Assembler, AssemblyContext, ProcedureCache},
    ast::{Form, FullyQualifiedProcedureName, Module, ModuleKind},
    diagnostics::{
        reporting::{set_hook, ReportHandlerOpts},
        Report, SourceFile,
    },
    Compile, CompileOpts, Library, LibraryPath, RpoDigest,
};

#[cfg(feature = "std")]
use crate::diagnostics::reporting::set_panic_hook;

use alloc::{boxed::Box, string::String, sync::Arc, vec::Vec};
use core::fmt;
use vm_core::{utils::DisplayHex, Program};

/// Represents a pattern for matching text abstractly
/// for use in asserting contents of complex diagnostics
#[derive(Debug)]
pub enum Pattern {
    /// Searches for an exact match of the given literal in the input string
    Literal(alloc::borrow::Cow<'static, str>),
    /// Searches for a match of the given regular expression in the input string
    Regex(regex::Regex),
}
impl Pattern {
    /// Construct a [Pattern] representing the given regular expression
    #[track_caller]
    pub fn regex(pattern: impl AsRef<str>) -> Self {
        Self::Regex(regex::Regex::new(pattern.as_ref()).expect("invalid regex"))
    }

    /// Check if this pattern matches `input`
    pub fn is_match(&self, input: impl AsRef<str>) -> bool {
        match self {
            Self::Literal(pattern) => input.as_ref().contains(pattern.as_ref()),
            Self::Regex(ref regex) => regex.is_match(input.as_ref()),
        }
    }

    /// Assert that this pattern matches `input`.
    ///
    /// This behaves like `assert_eq!` or `assert_matches!`, i.e. it
    /// will produce a helpful panic message on failure that renders
    /// the difference between what the pattern expected, and what
    /// it actually was matched against.
    #[track_caller]
    pub fn assert_match(&self, input: impl AsRef<str>) {
        let input = input.as_ref();
        if !self.is_match(input) {
            panic!(
                r"expected string was not found in emitted diagnostics:
expected input to {expected}
matched against: `{actual}`
",
                expected = self,
                actual = input
            );
        }
    }

    /// Like [Pattern::assert_match], but renders additional context
    /// in the case of failure to aid in troubleshooting.
    #[track_caller]
    pub fn assert_match_with_context(&self, input: impl AsRef<str>, context: impl AsRef<str>) {
        let input = input.as_ref();
        let context = context.as_ref();
        if !self.is_match(input) {
            panic!(
                r"expected string was not found in emitted diagnostics:
expected input to {expected}
matched against: `{actual}`
full output: `{context}`
",
                expected = self,
                actual = input
            );
        }
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Literal(ref lit) => write!(f, "contain `{}`", lit),
            Self::Regex(ref pat) => write!(f, "match regular expression `{}`", pat.as_str()),
        }
    }
}

impl From<&'static str> for Pattern {
    fn from(s: &'static str) -> Self {
        Self::Literal(alloc::borrow::Cow::Borrowed(s.trim()))
    }
}

impl From<String> for Pattern {
    fn from(s: String) -> Self {
        Self::Literal(alloc::borrow::Cow::Owned(s))
    }
}

impl From<regex::Regex> for Pattern {
    fn from(pat: regex::Regex) -> Self {
        Self::Regex(pat)
    }
}

/// Create a [Pattern::Regex] from the given input
#[macro_export]
macro_rules! regex {
    ($source:literal) => {
        Pattern::regex($source)
    };

    ($source:expr) => {
        Pattern::regex($source)
    };
}

/// Construct an [`Arc<SourceFile>`] from a string literal or expression,
/// such that emitted diagnostics reference the file and line on which
/// the source file was constructed.
#[macro_export]
macro_rules! source_file {
    ($source:literal) => {
        ::alloc::sync::Arc::new($crate::diagnostics::SourceFile::new(
            concat!("test", line!()),
            $source.to_string(),
        ))
    };
    ($source:expr) => {
        ::alloc::sync::Arc::new($crate::diagnostics::SourceFile::new(
            concat!("test", line!()),
            $source,
        ))
    };
}

/// Assert that the given diagnostic/error value, when rendered to stdout,
/// contains the given pattern
#[macro_export]
macro_rules! assert_diagnostic {
    ($diagnostic:expr, $expected:literal) => {{
        let actual = format!("{}", PrintDiagnostic::new_without_color($diagnostic));
        Pattern::from($expected).assert_match(actual);
    }};

    ($diagnostic:expr, $expected:expr) => {{
        let actual = format!("{}", PrintDiagnostic::new_without_color($diagnostic));
        Pattern::from($expected).assert_match(actual);
    }};
}

/// Like [assert_diagnostic], but matches each non-empty line of the rendered output
/// to a corresponding pattern. So if the output has 3 lines, the second of which is
/// empty, and you provide 2 patterns, the assertion passes if the first line matches
/// the first pattern, and the third line matches the second pattern - the second
/// line is ignored because it is empty.
#[macro_export]
macro_rules! assert_diagnostic_lines {
    ($diagnostic:expr, $($expected:expr),+) => {{
        let actual = format!("{}", $crate::diagnostics::reporting::PrintDiagnostic::new_without_color($diagnostic));
        let lines = actual.lines().filter(|l| !l.trim().is_empty()).zip([$(Pattern::from($expected)),*].into_iter());
        for (actual_line, expected) in lines {
            expected.assert_match_with_context(actual_line, &actual);
        }
    }};
}

/// A [TestContext] provides common functionality for all tests which interact with an [Assembler].
///
/// It is used by constructing it with `TestContext::default()`, which will initialize the
/// diagnostic reporting infrastructure, and construct a default [Assembler] instance for you. You
/// can then optionally customize the context, or start invoking any of its test helpers.
///
/// Some of the assertion macros defined above require a [TestContext], so be aware of that.
pub struct TestContext {
    assembler: Assembler,
}

impl Default for TestContext {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContext {
    pub fn new() -> Self {
        #[cfg(feature = "std")]
        {
            let result = set_hook(Box::new(|_| Box::new(ReportHandlerOpts::new().build())));
            #[cfg(feature = "std")]
            if result.is_ok() {
                set_panic_hook();
            }
        }

        #[cfg(not(feature = "std"))]
        {
            let _ = set_hook(Box::new(|_| Box::new(ReportHandlerOpts::new().build())));
        }
        Self {
            assembler: Assembler::default().with_debug_mode(true),
        }
    }

    /// Parse the given source file into a vector of top-level [Form]s.
    ///
    /// This does not run semantic analysis, or construct a [Module] from the parsed
    /// forms, and is largely intended for low-level testing of the parser.
    #[track_caller]
    pub fn parse_forms(&mut self, source: Arc<SourceFile>) -> Result<Vec<Form>, Report> {
        crate::parser::parse_forms(source.clone())
            .map_err(|err| Report::new(err).with_source_code(source))
    }

    /// Parse the given source file into an executable [Module].
    ///
    /// This runs semantic analysis, and the returned module is guaranteed to be syntactically
    /// valid.
    #[track_caller]
    pub fn parse_program(&mut self, source: impl Compile) -> Result<Box<Module>, Report> {
        source.compile()
    }

    /// Parse the given source file into a kernel [Module].
    ///
    /// This runs semantic analysis, and the returned module is guaranteed to be syntactically
    /// valid.
    #[allow(unused)]
    #[track_caller]
    pub fn parse_kernel(&mut self, source: impl Compile) -> Result<Box<Module>, Report> {
        source.compile_with_opts(CompileOpts::for_kernel())
    }

    /// Parse the given source file into an anonymous library [Module].
    ///
    /// This runs semantic analysis, and the returned module is guaranteed to be syntactically
    /// valid.
    #[track_caller]
    pub fn parse_module(&mut self, source: impl Compile) -> Result<Box<Module>, Report> {
        source.compile_with_opts(CompileOpts::for_library())
    }

    /// Parse the given source file into a library [Module] with the given fully-qualified path.
    #[track_caller]
    pub fn parse_module_with_path(
        &mut self,
        path: LibraryPath,
        source: impl Compile,
    ) -> Result<Box<Module>, Report> {
        source.compile_with_opts(CompileOpts::new(ModuleKind::Library, path).unwrap())
    }

    /// Add `module` to the [Assembler] constructed by this context, making it available to
    /// other modules.
    #[track_caller]
    pub fn add_module(&mut self, module: impl Compile) -> Result<(), Report> {
        self.assembler.add_module(module)
    }

    /// Add a module to the [Assembler] constructed by this context, with the fully-qualified
    /// name `path`, by parsing it from the provided source file.
    ///
    /// This will fail if the module cannot be parsed, fails semantic analysis, or conflicts
    /// with a previously added module within the assembler.
    #[track_caller]
    pub fn add_module_from_source(
        &mut self,
        path: LibraryPath,
        source: impl Compile,
    ) -> Result<(), Report> {
        self.assembler.add_module_with_options(
            source,
            CompileOpts {
                path: Some(path),
                ..CompileOpts::for_library()
            },
        )
    }

    /// Add the modules of `library` to the [Assembler] constructed by this context.
    #[track_caller]
    pub fn add_library<L>(&mut self, library: &L) -> Result<(), Report>
    where
        L: ?Sized + Library + 'static,
    {
        self.assembler.add_library(library)
    }

    /// Compile a [Program] from `source` using the [Assembler] constructed by this context.
    ///
    /// NOTE: Any modules added by, e.g. `add_module`, will be available to the executable
    /// module represented in `source`.
    #[track_caller]
    pub fn assemble(&mut self, source: impl Compile) -> Result<Program, Report> {
        self.assembler.assemble(source)
    }

    /// Compile a module from `source`, with the fully-qualified name `path`, to MAST, returning
    /// the MAST roots of all the exported procedures of that module.
    #[track_caller]
    pub fn assemble_module(
        &mut self,
        path: LibraryPath,
        module: impl Compile,
    ) -> Result<Vec<RpoDigest>, Report> {
        let mut context = AssemblyContext::for_library(&path);
        let options = CompileOpts {
            path: Some(path),
            ..CompileOpts::for_library()
        };
        self.assembler.assemble_module(module, options, &mut context)
    }

    /// Get a reference to the [ProcedureCache] of the [Assembler] constructed by this context.
    pub fn procedure_cache(&self) -> &ProcedureCache {
        self.assembler.procedure_cache()
    }

    /// Display the MAST root associated with `name` in the procedure cache of the [Assembler]
    /// constructed by this context.
    ///
    /// It is expected that the module containing `name` was previously compiled by the assembler,
    /// and is thus in the cache. This function will panic if that is not the case.
    pub fn display_digest_from_cache(
        &self,
        name: &FullyQualifiedProcedureName,
    ) -> impl fmt::Display {
        self.procedure_cache()
            .get_by_name(name)
            .map(|p| p.code().hash())
            .map(DisplayDigest)
            .unwrap_or_else(|| panic!("procedure '{}' is not in the procedure cache", name))
    }
}

struct DisplayDigest(RpoDigest);
impl fmt::Display for DisplayDigest {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:#x}", DisplayHex(self.0.as_bytes().as_slice()))
    }
}
